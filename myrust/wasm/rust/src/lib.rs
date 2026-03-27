use wasm_bindgen::prelude::*;

// Import JS console.log
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

/// A macro for convenient console logging
macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

// ============================================================
// Example 1: Basic string operations
// ============================================================

/// Greet someone by name
#[wasm_bindgen]
pub fn greet(name: &str) -> String {
    console_log!("greet() called with name: {}", name);
    format!("Hello, {}! Welcome to Rust + WebAssembly 🦀🕸️", name)
}

// ============================================================
// Example 2: Fibonacci calculation (CPU-intensive task)
// ============================================================

/// Calculate the nth Fibonacci number
#[wasm_bindgen]
pub fn fibonacci(n: u32) -> u64 {
    console_log!("fibonacci({}) called", n);
    match n {
        0 => 0,
        1 => 1,
        _ => {
            let mut a: u64 = 0;
            let mut b: u64 = 1;
            for _ in 2..=n {
                let temp = a + b;
                a = b;
                b = temp;
            }
            b
        }
    }
}

// ============================================================
// Example 3: Canvas drawing
// ============================================================

/// Draw a colorful pattern on a canvas element
#[wasm_bindgen]
pub fn draw_pattern(canvas_id: &str) -> Result<(), JsValue> {
    let document = web_sys::window()
        .ok_or("no window")?
        .document()
        .ok_or("no document")?;

    let canvas = document
        .get_element_by_id(canvas_id)
        .ok_or("canvas not found")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    let ctx = canvas
        .get_context("2d")?
        .ok_or("no 2d context")?
        .dyn_into::<web_sys::CanvasRenderingContext2d>()?;

    let width = canvas.width() as f64;
    let height = canvas.height() as f64;

    // Clear canvas
    ctx.clear_rect(0.0, 0.0, width, height);

    // Draw concentric circles with gradient colors
    let center_x = width / 2.0;
    let center_y = height / 2.0;
    let max_radius = width.min(height) / 2.0 - 10.0;
    let num_circles = 12;

    for i in (0..num_circles).rev() {
        let ratio = (i as f64 + 1.0) / num_circles as f64;
        let radius = max_radius * ratio;

        // Generate rainbow colors using HSL
        let hue = (i as f64 / num_circles as f64) * 360.0;
        let color = format!("hsl({}, 80%, 60%)", hue);

        ctx.begin_path();
        ctx.arc(center_x, center_y, radius, 0.0, std::f64::consts::PI * 2.0)?;
        ctx.set_fill_style_str(&color);
        ctx.fill();
    }

    // Draw a Rust logo text in the center
    ctx.set_fill_style_str("#ffffff");
    ctx.set_font("bold 24px Arial");
    ctx.set_text_align("center");
    ctx.set_text_baseline("middle");
    ctx.fill_text("🦀 Rust WASM", center_x, center_y)?;

    console_log!("Pattern drawn on canvas '{}'", canvas_id);
    Ok(())
}

// ============================================================
// Example 4: DOM manipulation
// ============================================================

/// Create a styled element and append it to a container
#[wasm_bindgen]
pub fn add_item_to_list(container_id: &str, text: &str) -> Result<(), JsValue> {
    let document = web_sys::window()
        .ok_or("no window")?
        .document()
        .ok_or("no document")?;

    let container = document
        .get_element_by_id(container_id)
        .ok_or("container not found")?;

    let item = document.create_element("div")?;
    item.set_class_name("list-item");
    item.set_inner_html(&format!("🦀 {}", text));

    container.append_child(&item)?;

    console_log!("Added item '{}' to container '{}'", text, container_id);
    Ok(())
}

// ============================================================
// Example 5: Array processing
// ============================================================

/// Sort an array of numbers and return the sorted result as a string
#[wasm_bindgen]
pub fn sort_numbers(input: &str) -> String {
    let mut numbers: Vec<f64> = input
        .split(',')
        .filter_map(|s| s.trim().parse().ok())
        .collect();

    numbers.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

    console_log!("Sorted {} numbers", numbers.len());

    numbers
        .iter()
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(", ")
}
