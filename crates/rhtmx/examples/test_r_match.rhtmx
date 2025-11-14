// Quick test to demonstrate r-match working with let bindings

use rhtmx::{html, Html};

#[derive(Debug, Clone)]
enum UserStatus {
    Active,
    Pending,
    Suspended,
}

fn main() {
    println!("=== Testing r-match in let bindings ===\n");

    // Test 1: Active status
    let status = UserStatus::Active;
    let badge = html! {
        <div r-match="status">
            <span r-when="UserStatus::Active">"✓ Active"</span>
            <span r-when="UserStatus::Pending">"⏳ Pending"</span>
            <span r-when="UserStatus::Suspended">"❌ Suspended"</span>
            <span r-default>"Unknown"</span>
        </div>
    };
    println!("Active status: {}", badge.0);

    // Test 2: Pending status
    let status = UserStatus::Pending;
    let badge = html! {
        <div r-match="status">
            <span r-when="UserStatus::Active">"✓ Active"</span>
            <span r-when="UserStatus::Pending">"⏳ Pending"</span>
            <span r-when="UserStatus::Suspended">"❌ Suspended"</span>
            <span r-default>"Unknown"</span>
        </div>
    };
    println!("Pending status: {}", badge.0);

    // Test 3: Nested in function
    fn render_status(status: UserStatus) -> Html {
        html! {
            <div class="status-badge" r-match="status">
                <span r-when="UserStatus::Active" class="active">"✓ Active"</span>
                <span r-when="UserStatus::Pending" class="pending">"⏳ Pending"</span>
                <span r-default>"Unknown"</span>
            </div>
        }
    }

    println!(
        "Function result: {}",
        render_status(UserStatus::Suspended).0
    );

    println!("\n✅ All r-match tests passed!");
}
