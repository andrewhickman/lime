mod geom;
mod tri;

pub use self::geom::Point;

use std::sync::Arc;

use failure::Fallible;
use rusttype::PositionedGlyph;
use utils::throw;
use vulkano::command_buffer::{AutoCommandBufferBuilder, DynamicState};
use vulkano::device::Device;
use vulkano::framebuffer::{RenderPassAbstract, Subpass};
use vulkano_glyph::{FontId, GlyphBrush, Section as GlyphSection};

use d2::tri::{TriangleBrush, TriangleSection};
use Color;

pub struct Renderer {
    tri_brush: TriangleBrush,
    glyph_brush: GlyphBrush<'static>,
    sections: Vec<Section>,
}

enum Section {
    Triangle(TriangleSection),
    Glyph(GlyphSection),
}

impl Renderer {
    pub(crate) fn new(
        device: &Arc<Device>,
        subpass: Subpass<Arc<RenderPassAbstract + Send + Sync>>,
    ) -> Self {
        let tri_brush = TriangleBrush::new(device, subpass.clone());
        let glyph_brush = GlyphBrush::new(device, subpass).unwrap_or_else(throw);
        Renderer {
            tri_brush,
            glyph_brush,
            sections: Vec::new(),
        }
    }

    pub(crate) fn commit(
        &mut self,
        mut cmd: AutoCommandBufferBuilder,
        state: &DynamicState,
        logical_size: [f32; 2],
    ) -> Fallible<AutoCommandBufferBuilder> {
        for section in self.sections.drain(..) {
            match section {
                Section::Triangle(section) => {
                    cmd = self.tri_brush.draw(cmd, &section, state, logical_size)?;
                }
                Section::Glyph(section) => {
                    cmd = self.glyph_brush.draw(
                        cmd,
                        &section,
                        state,
                        [
                            [1.0, 0.0, 0.0, 0.0],
                            [0.0, 1.0, 0.0, 0.0],
                            [0.0, 0.0, 1.0, 0.0],
                            [0.0, 0.0, 0.0, 1.0],
                        ],
                        logical_size,
                    )?;
                }
            }
        }

        self.glyph_brush.clear();

        Ok(cmd)
    }

    pub fn draw_tris(&mut self, vertices: &[Point], color: Color) {
        let subsection = self.tri_brush.queue_tris(vertices, color);
        if let Some(Section::Triangle(section)) = self.sections.last_mut() {
            section.append(&subsection);
            return;
        }
        self.sections.push(Section::Triangle(subsection));
    }

    pub fn draw_glyphs<I>(&mut self, glyphs: I, font: FontId, color: Color)
    where
        I: IntoIterator<Item = PositionedGlyph<'static>>,
    {
        let section = self.glyph_brush.queue_glyphs(glyphs, font, color.into());
        self.sections.push(Section::Glyph(section));
    }
}
