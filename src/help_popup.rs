use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Block, Borders, Paragraph, Widget},
};

#[cfg(test)]
const MARGIN_PERCENT: u16 = 0;
#[cfg(not(test))]
const MARGIN_PERCENT: u16 = 15;

pub struct HelpPopup;

impl Widget for HelpPopup {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let margin_x = area.width * MARGIN_PERCENT / 100 / 2;
        let margin_y = area.height * MARGIN_PERCENT / 100;
        let popup_area = Rect {
            x: area.x + margin_x,
            y: area.y + margin_y,
            width: area.width.saturating_sub(margin_x * 2),
            height: area.height.saturating_sub(margin_y * 2),
        };
        // Clear the popup area
        for y in popup_area.y..popup_area.y + popup_area.height {
            for x in popup_area.x..popup_area.x + popup_area.width {
                if x < area.width && y < area.height {
                    buf[(x, y)].reset();
                }
            }
        }
        let block = Block::default()
            .borders(Borders::ALL)
            .border_set(ratatui::symbols::border::PLAIN)
            .title(" Help ");
        let inner = block.inner(popup_area);
        block.render(popup_area, buf);
        Paragraph::new("hello world").render(inner, buf);
    }
}
