use ratatui::text::Text;
use ratatui::widgets::{Block, Borders, Clear, Paragraph};
use ratatui::Frame;

use crate::ui::{centered_rect, style};

const HELP: &str = r#" 全局
   q / Ctrl+C  退出        ?      帮助
   Tab         切换面板    1-4    焦点直达
   t           折叠终端    r      刷新
   b           分支        P      推送  F  拉取

 变更（左栏 · 变更）
   j/k         移动        space  暂存/取消
   Enter       查看差异    a      全部暂存
   c           提交        u      取消暂存

 差异（中栏）
   v           左右/统一   n/N    上一个/下一个差异点
   j/k         滚动

 文件（左栏 · 文件）
   Enter/l     展开/打开   h      收起
   e           编辑        Ctrl+S 保存  Esc 退出编辑

 历史（右栏）
   Enter       查看提交文件  Enter(文件) 查看 patch

 终端
   i           输入命令    Ctrl+C 中断
"#;

pub fn draw(f: &mut Frame) {
    let area = f.area();
    let popup = centered_rect(70, 85, area);
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" 帮助（Esc/q 关闭） ")
        .border_style(style::border(true));
    let para = Paragraph::new(Text::from(HELP)).block(block);
    f.render_widget(Clear, popup);
    f.render_widget(para, popup);
}
