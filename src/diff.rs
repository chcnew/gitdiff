use similar::{ChangeTag, TextDiff};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiffTag {
    Equal,
    Add,
    Del,
    Mod,
}

#[derive(Debug, Clone)]
pub struct DiffRow {
    pub tag: DiffTag,
    pub left: Option<String>,
    pub right: Option<String>,
}

/// 将两侧文本对齐为 side-by-side 行。
///
/// 相邻的删除 + 新增合并为「修改（Mod）」，其余为纯增/删。
pub fn side_by_side(left: &str, right: &str, max_lines: usize) -> Vec<DiffRow> {
    let diff = TextDiff::from_lines(left, right);
    let mut rows: Vec<DiffRow> = Vec::new();
    let mut dels: Vec<String> = Vec::new();
    let mut ins: Vec<String> = Vec::new();

    for op in diff.ops() {
        for change in diff.iter_changes(op) {
            let value = clean(change.value());
            match change.tag() {
                ChangeTag::Equal => {
                    flush(&mut dels, &mut ins, &mut rows);
                    rows.push(DiffRow {
                        tag: DiffTag::Equal,
                        left: Some(value.clone()),
                        right: Some(value),
                    });
                }
                ChangeTag::Delete => dels.push(value),
                ChangeTag::Insert => ins.push(value),
            }
        }
    }
    flush(&mut dels, &mut ins, &mut rows);

    if rows.len() > max_lines {
        rows.truncate(max_lines);
    }
    rows
}

fn flush(dels: &mut Vec<String>, ins: &mut Vec<String>, rows: &mut Vec<DiffRow>) {
    let n = dels.len().max(ins.len());
    for i in 0..n {
        let l = dels.get(i).cloned();
        let r = ins.get(i).cloned();
        let tag = if l.is_some() && r.is_some() {
            DiffTag::Mod
        } else if l.is_some() {
            DiffTag::Del
        } else {
            DiffTag::Add
        };
        rows.push(DiffRow {
            tag,
            left: l,
            right: r,
        });
    }
    dels.clear();
    ins.clear();
}

fn clean(s: &str) -> String {
    s.trim_end_matches('\n').trim_end_matches('\r').to_string()
}
