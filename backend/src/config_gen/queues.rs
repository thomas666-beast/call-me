use crate::models::queue::Queue;

const HEADER: &str = "\
; ============================================================================
; queues.conf — GENERATED FILE
; ============================================================================
;
; Regenerated every time a queue or its members change.
; Manual edits will be overwritten on the next save.
; ============================================================================

[general]
persistentmembers = yes
autofill = yes
shared_lastcall = no
";

/// Render the full queues.conf from the given queues.
///
/// `members` is a parallel list: `members[i]` is the list of extension
/// numbers belonging to `queues[i]`.
pub fn render(queues: &[Queue], members: &[Vec<String>]) -> String {
    let mut out = String::from(HEADER);
    out.push('\n');

    if queues.is_empty() {
        out.push_str("; (no queues configured yet)\n");
        return out;
    }

    for (queue, member_list) in queues.iter().zip(members.iter()) {
        if !queue.active {
            out.push_str(&format!(
                "; Queue {} ({}) is disabled\n\n",
                queue.number, queue.name
            ));
            continue;
        }
        out.push_str(&render_one(queue, member_list));
        out.push('\n');
    }

    out
}

fn render_one(queue: &Queue, members: &[String]) -> String {
    let mut s = format!("; Queue {} — {}\n[{}]\n", queue.number, queue.name, queue.number);

    s.push_str(&format!("strategy={}\n", queue.strategy));
    s.push_str(&format!("timeout={}\n", queue.timeout));
    s.push_str(&format!("retry={}\n", queue.retry));
    s.push_str(&format!("maxlen={}\n", queue.maxlen));
    s.push_str(&format!("musicclass={}\n", queue.musicclass));
    s.push_str(&format!("joinempty={}\n", yesno(queue.joinempty)));
    s.push_str(&format!("leavewhenempty={}\n", yesno(queue.leavewhenempty)));
    s.push_str(&format!("ringinuse={}\n", yesno(queue.ringinuse)));

    for ext in members {
        s.push_str(&format!("member => PJSIP/{}\n", ext));
    }

    s
}

fn yesno(b: bool) -> &'static str {
    if b { "yes" } else { "no" }
}
