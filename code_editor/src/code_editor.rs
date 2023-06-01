use {
    crate::{state::ViewId, State},
    makepad_widgets::*,
};

live_design! {
    import makepad_widgets::theme::*;

    CodeEditor = {{CodeEditor}} {
        walk: {
            width: Fill,
            height: Fill,
            margin: 0,
        },
        draw_text: {
            draw_depth: 0.0,
            text_style: <FONT_CODE> {},
        }
    }
}

#[derive(Live, LiveHook)]
pub struct CodeEditor {
    #[live]
    walk: Walk,
    #[live]
    scroll_bars: ScrollBars,
    #[live]
    draw_text: DrawText,
}

impl CodeEditor {
    pub fn draw(&mut self, cx: &mut Cx2d<'_>, state: &State, view_id: ViewId) {
        use crate::StrExt;

        self.scroll_bars.begin(cx, self.walk, Layout::default());
        let scroll_position = self.scroll_bars.get_scroll_pos();
        let glyph_size =
            self.draw_text.text_style.font_size * self.draw_text.get_monospace_base(cx);
        let mut max_line_size_x = 0.0;
        let mut position = DVec2::new();
        for line in state.context(view_id).lines() {
            for token in line.tokens() {
                self.draw_text
                    .draw_abs(cx, position - scroll_position, token.text);
                position.x += token.text.graphemes().count() as f64 * glyph_size.x;
            }
            max_line_size_x = max_line_size_x.max(position.x);
            position.x = 0.0;
            position.y += glyph_size.y;
        }
        cx.turtle_mut().set_used(max_line_size_x, position.y);
        self.scroll_bars.end(cx);
    }

    pub fn handle_event(&mut self, cx: &mut Cx, event: &Event) {
        self.scroll_bars.handle_event_with(cx, event, &mut |cx, _| {
            cx.redraw_all();
        });
    }
}
