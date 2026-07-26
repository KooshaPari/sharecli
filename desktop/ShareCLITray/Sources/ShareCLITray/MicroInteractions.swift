/// MicroInteractions.swift — small reusable view modifiers for dashboard polish.
///
/// Provides three building blocks that the dashboard pages compose on top of
/// their existing layouts:
///
///  1. `.animateInOnAppear(delay:)` — scale+opacity entrance animation
///     for summary cards, list rows, and panel headers. Designed to feel
///     like the page "settles" rather than pop in.
///
///  2. `PressableButtonStyle` — a button style that scales the content to
///     0.96 with a 0.08s spring on press and back to 1.0 on release.
///     Drop-in replacement for `.borderless` / `.plain` button styles
///     used on action buttons across the dashboard.
///
///  3. `.hoverGlow(radius:)` — a subtle accent-colored border + drop shadow
///     that fades in when the cursor hovers over a card. Pairs naturally
///     with the existing `.background(.quaternary)` cards.
///
///  All three are wired up so that SwiftUI's `Label`s + `Image`s + `Text`s
///  render unchanged when the modifiers are not applied — they only add
///  visual signal on interaction.
import SwiftUI

// MARK: - Entrance animation

private struct AnimateInOnAppearModifier: ViewModifier {
    let delay: Double
    @State private var visible: Bool = false

    func body(content: Content) -> some View {
        content
            .opacity(visible ? 1.0 : 0.0)
            .scaleEffect(visible ? 1.0 : 0.97)
            .onAppear {
                withAnimation(.spring(response: 0.42, dampingFraction: 0.78).delay(delay)) {
                    visible = true
                }
            }
    }
}

extension View {
    /// Fades + scales the view in when it first appears. Stagger with `delay`
    /// to cascade summary cards, list rows, and panel headers.
    func animateInOnAppear(delay: Double = 0) -> some View {
        modifier(AnimateInOnAppearModifier(delay: delay))
    }
}

// MARK: - Press feedback

/// A button style that scales content to 0.96 on press and back to 1.0
/// on release with a quick spring. Pairs with `.borderless` look — no
/// default chrome change, just tactile feedback.
struct PressableButtonStyle: ButtonStyle {
    func makeBody(configuration: Configuration) -> some View {
        configuration.label
            .scaleEffect(configuration.isPressed ? 0.96 : 1.0)
            .opacity(configuration.isPressed ? 0.85 : 1.0)
            .animation(.spring(response: 0.18, dampingFraction: 0.7), value: configuration.isPressed)
    }
}

extension ButtonStyle where Self == PressableButtonStyle {
    static var pressable: PressableButtonStyle { PressableButtonStyle() }
}

// MARK: - Hover glow

private struct HoverGlowModifier: ViewModifier {
    let radius: CGFloat
    @State private var hovering: Bool = false

    func body(content: Content) -> some View {
        content
            .overlay(
                RoundedRectangle(cornerRadius: 8)
                    .strokeBorder(Color.accentColor.opacity(hovering ? 0.5 : 0.0), lineWidth: 1.5)
                    .shadow(color: Color.accentColor.opacity(hovering ? 0.18 : 0.0), radius: radius)
                    .animation(.easeInOut(duration: 0.18), value: hovering)
                    .allowsHitTesting(false)
            )
            .onHover { hovering = $0 }
    }
}

extension View {
    /// Accent-colored glow on hover. Best on cards that already use
    /// `.background(.quaternary).clipShape(RoundedRectangle(cornerRadius: 8))`.
    func hoverGlow(radius: CGFloat = 6) -> some View {
        modifier(HoverGlowModifier(radius: radius))
    }
}

// MARK: - Smooth value-transition for numeric data

private struct AnimatedNumberModifier: ViewModifier {
    let value: Double
    let formatter: (Double) -> String
    @State private var displayed: Double = 0

    func body(content: Content) -> some View {
        Text(formatter(displayed))
            .contentTransition(.numericText())
            .onAppear { displayed = value }
            .onChange(of: value) { _, newValue in
                withAnimation(.easeOut(duration: 0.32)) {
                    displayed = newValue
                }
            }
    }
}

extension View {
    /// Animates a numeric value through a custom formatter. Use for
    /// counters in summary cards (e.g. "12" → "13" with a small tween).
    func animatedNumber(_ value: Double, formatter: @escaping (Double) -> String) -> some View {
        modifier(AnimatedNumberModifier(value: value, formatter: formatter))
    }
}
