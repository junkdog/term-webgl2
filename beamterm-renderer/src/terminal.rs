use beamterm_data::FontAtlasData;
use compact_str::CompactString;
use web_sys::HtmlCanvasElement;

use crate::{CellData, Error, FontAtlas, Renderer, TerminalGrid};

/// High-performance WebGL2 terminal renderer.
///
/// `Terminal` encapsulates the complete terminal rendering system, providing a
/// simplified API over the underlying [`Renderer`] and [`TerminalGrid`] components.
///
/// # Examples
///
/// ```rust
/// use beamterm_renderer::{CellData, Terminal};
///
/// // Create and render a simple terminal
/// let mut terminal = Terminal::builder("#canvas").build()?;
///
/// // Update cells with content
/// let cells: Vec<CellData> = unimplemented!();
/// terminal.update_cells(cells.into_iter())?;
///
/// // Render frame
/// terminal.render_frame()?;
///
/// // Handle window resize
/// let (new_width, new_height) = (800, 600);
/// terminal.resize(new_width, new_height)?;
/// ```
pub struct Terminal {
    renderer: Renderer,
    grid: TerminalGrid,
}

impl Terminal {
    /// Creates a new terminal builder with the specified canvas source.
    ///
    /// # Parameters
    /// * `canvas` - Canvas identifier (CSS selector) or `HtmlCanvasElement`
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Using CSS selector
    /// use web_sys::HtmlCanvasElement;
    /// use beamterm_renderer::Terminal;
    /// let terminal = Terminal::builder("my-terminal").build()?;
    ///
    /// // Using canvas element
    /// let canvas: &HtmlCanvasElement = unimplemented!("document.get_element_by_id(...)");
    /// let terminal = Terminal::builder(canvas).build()?;
    /// ```
    #[allow(private_bounds)]
    pub fn builder(canvas: impl Into<CanvasSource>) -> TerminalBuilder {
        TerminalBuilder::new(canvas.into())
    }

    /// Updates terminal cell content efficiently.
    ///
    /// This method batches all cell updates and uploads them to the GPU in a single
    /// operation. For optimal performance, collect all changes and update in one call
    /// rather than making multiple calls for individual cells.
    ///
    /// Delegates to [`TerminalGrid::update_cells`].
    pub fn update_cells<'a>(
        &mut self,
        cells: impl Iterator<Item = CellData<'a>>,
    ) -> Result<(), Error> {
        self.grid.update_cells(self.renderer.gl(), cells)?;
        Ok(())
    }

    /// Returns the WebGL2 rendering context.
    pub fn gl(&self) -> &web_sys::WebGl2RenderingContext {
        self.renderer.gl()
    }

    /// Resizes the terminal to fit new canvas dimensions.
    ///
    /// This method updates both the renderer viewport and terminal grid to match
    /// the new canvas size. The terminal dimensions (in cells) are automatically
    /// recalculated based on the cell size from the font atlas.
    ///
    /// Combines [`Renderer::resize`] and [`TerminalGrid::resize`] operations.
    pub fn resize(&mut self, width: i32, height: i32) -> Result<(), Error> {
        self.renderer.resize(width, height);
        self.grid.resize(self.renderer.gl(), (width, height))
    }

    /// Returns the font atlas used by the terminal.
    pub fn atlas(&self) -> &FontAtlas {
        self.grid.atlas()
    }

    /// Returns the terminal dimensions in cells.
    pub fn terminal_size(&self) -> (u16, u16) {
        self.grid.terminal_size()
    }

    /// Returns the size of the canvas in pixels.
    pub fn canvas_size(&self) -> (i32, i32) {
        self.renderer.canvas_size()
    }

    /// Returns the size of each cell in pixels.
    pub fn cell_size(&self) -> (i32, i32) {
        self.grid.cell_size()
    }

    /// Returns a reference to the HTML canvas element used for rendering.
    pub fn canvas(&self) -> &HtmlCanvasElement {
        self.renderer.canvas()
    }

    /// Returns a reference to the underlying renderer.
    pub fn renderer(&self) -> &Renderer {
        &self.renderer
    }

    /// Returns a reference to the terminal grid.
    pub fn grid(&self) -> &TerminalGrid {
        &self.grid
    }

    /// Renders the current terminal state to the canvas.
    ///
    /// This method performs the complete render pipeline: frame setup, grid rendering,
    /// and frame finalization. Call this after updating terminal content to display
    /// the changes.
    ///
    /// Combines [`Renderer::begin_frame`], [`Renderer::render`], and [`Renderer::end_frame`].
    ///
    pub fn render_frame(&mut self) -> Result<(), Error> {
        self.renderer.begin_frame();
        self.renderer.render(&self.grid);
        self.renderer.end_frame();
        Ok(())
    }
}

/// Canvas source for terminal initialization.
///
/// Supports both CSS selector strings and direct `HtmlCanvasElement` references
/// for flexible terminal creation.
enum CanvasSource {
    /// CSS selector string for canvas lookup (e.g., "#terminal", "canvas").
    Id(CompactString),
    /// Direct reference to an existing canvas element.
    Element(web_sys::HtmlCanvasElement),
}

/// Builder for configuring and creating a [`Terminal`].
///
/// Provides a fluent API for terminal configuration with sensible defaults.
/// The terminal will use the default embedded font atlas unless explicitly configured.
///
/// # Examples
///
/// ```rust
/// // Simple terminal with default configuration
/// use beamterm_data::FontAtlasData;
/// use beamterm_renderer::{FontAtlas, Terminal};
/// let terminal = Terminal::builder("#canvas").build()?;
///
/// // Terminal with custom font atlas
/// let atlas = FontAtlasData::from_binary(unimplemented!(".atlas data"))?;
/// let terminal = Terminal::builder("#canvas")
///     .font_atlas(atlas)
///     .fallback_glyph("X".into())
///     .build()?;
/// ```
pub struct TerminalBuilder {
    canvas: CanvasSource,
    atlas: Option<FontAtlasData>,
    fallback_glyph: Option<CompactString>,
}

impl TerminalBuilder {
    /// Creates a new terminal builder with the specified canvas source.
    fn new(canvas: CanvasSource) -> Self {
        TerminalBuilder {
            canvas,
            atlas: None,
            fallback_glyph: None,
        }
    }

    /// Sets a custom font atlas for the terminal.
    ///
    /// By default, the terminal uses an embedded font atlas. Use this method
    /// to provide a custom atlas with different fonts, sizes, or character sets.
    pub fn font_atlas(mut self, atlas: FontAtlasData) -> Self {
        self.atlas = Some(atlas);
        self
    }

    /// Sets the fallback glyph for missing characters.
    ///
    /// When a character is not found in the font atlas, this glyph will be
    /// displayed instead. Defaults to a space character if not specified.
    pub fn fallback_glyph(mut self, glyph: &str) -> Self {
        self.fallback_glyph = Some(glyph.into());
        self
    }

    /// Builds the terminal with the configured options.
    pub fn build(self) -> Result<Terminal, Error> {
        let renderer = match self.canvas {
            CanvasSource::Id(id) => Renderer::create(&id)?,
            CanvasSource::Element(element) => Renderer::create_with_canvas(element)?,
        };

        let gl = renderer.gl();
        let atlas = match self.atlas {
            Some(atlas_data) => FontAtlas::load(gl, atlas_data)?,
            None => FontAtlas::load_default(gl)?,
        };

        let canvas_size = renderer.canvas_size();
        let mut grid = TerminalGrid::new(gl, atlas, canvas_size)?;
        if let Some(fallback) = self.fallback_glyph {
            grid.set_fallback_glyph(&fallback)
        };

        Ok(Terminal { renderer, grid })
    }
}

impl From<&'static str> for CanvasSource {
    fn from(id: &'static str) -> Self {
        CanvasSource::Id(id.into())
    }
}

impl From<web_sys::HtmlCanvasElement> for CanvasSource {
    fn from(element: web_sys::HtmlCanvasElement) -> Self {
        CanvasSource::Element(element)
    }
}

impl<'a> From<&'a web_sys::HtmlCanvasElement> for CanvasSource {
    fn from(value: &'a HtmlCanvasElement) -> Self {
        value.clone().into()
    }
}
