use crate::os::session::Session;
use crate::terminal::ansi::Style;
use crate::terminal::table::{Table, Column};

pub fn whoami(session: &Session) -> String {
    session.vault.identity.handle.clone()
}

pub fn id(session: &Session) -> String {
    let ident = &session.vault.identity;
    format!(
        "uid=1000({}) gid=1000(engineers) groups=1000(engineers),1001(wheel)",
        ident.handle
    )
}

pub fn roles(session: &Session) -> String {
    let headers = &[
        Column { header: "ROLE", width: 28 },
        Column { header: "ORG", width: 16 },
        Column { header: "TENURE", width: 10 },
        Column { header: "FOCUS", width: 40 },
    ];
    let rows: Vec<Vec<String>> = session.vault.roles.iter().map(|r| {
        vec![
            Style::amber(&r.title),
            r.org.clone(),
            r.tenure.clone(),
            Style::dim(&r.focus),
        ]
    }).collect();
    let mut output = String::new();
    output.push_str(&Style::bold(&Style::amber("Roles & Responsibilities")));
    output.push_str("\r\n");
    output.push_str(&Style::rule(session.cols().unwrap_or(70)));
    output.push_str("\r\n");
    output.push_str(&Table::render(headers, &rows, session.cols()));
    output
}

pub fn stacks(session: &Session) -> String {
    let headers = &[
        Column { header: "TECHNOLOGY", width: 24 },
        Column { header: "PROFICIENCY", width: 14 },
        Column { header: "YEARS", width: 6 },
    ];
    let rows: Vec<Vec<String>> = session.vault.stacks.iter().map(|s| {
        let prof_color = match s.proficiency.as_str() {
            "expert" => Style::amber(&s.proficiency.to_uppercase()),
            "advanced" => Style::cyan(&s.proficiency),
            _ => s.proficiency.clone(),
        };
        vec![
            s.name.clone(),
            prof_color,
            s.years.to_string(),
        ]
    }).collect();
    let mut output = String::new();
    output.push_str(&Style::bold(&Style::amber("Technology Stacks")));
    output.push_str("\r\n");
    output.push_str(&Style::rule(session.cols().unwrap_or(70)));
    output.push_str("\r\n");
    // Fixed grid — the last column (YEARS) is short and shouldn't expand.
    output.push_str(&Table::render(headers, &rows, None));
    output
}

pub fn systems(session: &Session) -> String {
    let headers = &[
        Column { header: "SYSTEM", width: 32 },
        Column { header: "ARCHITECTURE", width: 28 },
        Column { header: "DESCRIPTION", width: 48 },
    ];
    let rows: Vec<Vec<String>> = session.vault.systems.iter().map(|s| {
        vec![
            Style::amber(&s.name),
            Style::dim(&s.architecture),
            s.description.clone(),
        ]
    }).collect();
    let mut output = String::new();
    output.push_str(&Style::bold(&Style::amber("Systems Designed & Built")));
    output.push_str("\r\n");
    output.push_str(&Style::rule(session.cols().unwrap_or(70)));
    output.push_str("\r\n");
    output.push_str(&Table::render(headers, &rows, session.cols()));
    output
}

pub fn projects(session: &Session) -> String {
    let headers = &[
        Column { header: "PROJECT", width: 30 },
        Column { header: "STATUS", width: 10 },
        Column { header: "DESCRIPTION", width: 50 },
    ];
    let rows: Vec<Vec<String>> = session.vault.projects.iter().map(|p| {
        let status_color = match p.status.as_str() {
            "active" => Style::green(&p.status.to_uppercase()),
            "inert" => Style::grey(&p.status.to_uppercase()),
            "frozen" => Style::cyan(&p.status.to_uppercase()),
            _ => p.status.clone(),
        };
        vec![
            Style::amber(&p.name),
            status_color,
            p.description.clone(),
        ]
    }).collect();
    let mut output = String::new();
    output.push_str(&Style::bold(&Style::amber("Projects")));
    output.push_str("\r\n");
    output.push_str(&Style::rule(session.cols().unwrap_or(70)));
    output.push_str("\r\n");
    output.push_str(&Table::render(headers, &rows, session.cols()));
    output
}
