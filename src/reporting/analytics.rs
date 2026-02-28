pub struct Analytics;

impl Analytics {
    pub fn render_bar_chart(data: &[(String, i32)]) -> String {
        let mut chart = String::new();
        let max_val = data.iter().map(|(_, v)| *v).max().unwrap_or(1);
        let max_label_len = data.iter().map(|(l, _)| l.len()).max().unwrap_or(0);

        for (label, value) in data {
            let bar_len = if max_val > 0 {
                ((*value as f32 / max_val as f32) * 20.0) as usize
            } else {
                0
            };
            let bar = "█".repeat(bar_len);
            chart.push_str(&format!(
                "{:width$} | {} ({})\n",
                label,
                bar,
                value,
                width = max_label_len
            ));
        }
        chart
    }
}
