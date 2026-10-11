use crate::models::extension::Extension;

/// Header written at the top of the generated file.
const HEADER: &str = "\
; ============================================================================
; pjsip_extensions.conf — GENERATED FILE
; ============================================================================
;
; Regenerated every time an extension is created, updated, or deleted.
; Manual edits will be overwritten.
; ============================================================================

";

/// Render the full contents of `pjsip_extensions.conf` from a list of
/// extensions. Only active extensions are emitted.
pub fn render(extensions: &[Extension]) -> String {
    let mut out = String::from(HEADER);

    if extensions.is_empty() {
        out.push_str("; (no extensions configured yet)\n");
        return out;
    }

    for ext in extensions.iter().filter(|e| e.active) {
        out.push_str(&render_one(ext));
        out.push('\n');
    }

    let inactive: Vec<&Extension> = extensions.iter().filter(|e| !e.active).collect();
    if !inactive.is_empty() {
        out.push_str("; Disabled extensions (not emitted):\n");
        for ext in inactive {
            out.push_str(&format!(";   - {} ({})\n", ext.number, ext.display_name));
        }
    }

    out
}

fn render_one(ext: &Extension) -> String {
    let codec_allow_lines: String = ext
        .codecs
        .iter()
        .map(|c| format!("allow={}\n", c))
        .collect();

    format!(
        "\
; Extension {number} — {display_name}
[{number}]
type=endpoint
transport={transport}
context={context}
disallow=all
{codecs}auth={number}
aors={number}
callerid={display_name} <{number}>
direct_media=no

[{number}]
type=auth
auth_type=userpass
username={number}
password={secret}

[{number}]
type=aor
max_contacts={max_contacts}
",
        number = ext.number,
        display_name = ext.display_name,
        transport = ext.transport,
        context = ext.context,
        codecs = codec_allow_lines,
        secret = ext.secret,
        max_contacts = ext.max_contacts,
    )
}
