package dev.pudding.ui

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Shapes
import androidx.compose.material3.Typography
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp

@Immutable
data class PuddingColors(
    val surface: Color, val subtle: Color, val hover: Color, val selected: Color,
    val text: Color, val secondary: Color, val muted: Color, val border: Color,
    val solid: Color, val inverse: Color, val accent: Color, val error: Color,
) {
    companion object {
        val Light = PuddingColors(Color(PuddingPalette.lightSurface), Color(PuddingPalette.lightSubtle), Color(PuddingPalette.lightHover), Color(PuddingPalette.lightSelected), Color(PuddingPalette.lightText), Color(PuddingPalette.lightSecondary), Color(PuddingPalette.lightMuted), Color(PuddingPalette.lightBorder), Color(PuddingPalette.lightSolid), Color(PuddingPalette.lightInverse), Color(PuddingPalette.lightAccent), Color(PuddingPalette.lightError))
        val Dark = PuddingColors(Color(PuddingPalette.darkSurface), Color(PuddingPalette.darkSubtle), Color(PuddingPalette.darkHover), Color(PuddingPalette.darkSelected), Color(PuddingPalette.darkText), Color(PuddingPalette.darkSecondary), Color(PuddingPalette.darkMuted), Color(PuddingPalette.darkBorder), Color(PuddingPalette.darkSolid), Color(PuddingPalette.darkInverse), Color(PuddingPalette.darkAccent), Color(PuddingPalette.darkError))
    }
}

@Immutable
data class PuddingDesign(
    val radius: Float = PuddingPalette.radius,
    val fontSize: Float = PuddingPalette.fontSize,
    val borderWidth: Float = PuddingPalette.borderWidth,
    val motionMillis: Int = PuddingPalette.motionMillis,
    val shadow: Boolean = PuddingPalette.shadow,
) {
    init {
        require(radius.isFinite() && radius in 0f..24f)
        require(fontSize.isFinite() && fontSize in 12f..24f)
        require(borderWidth.isFinite() && borderWidth in 0f..6f)
        require(motionMillis in 0..600)
    }
}

private val LocalColors = staticCompositionLocalOf { PuddingColors.Light }
private val LocalDesign = staticCompositionLocalOf { PuddingDesign() }

object PuddingTheme {
    val colors: PuddingColors @Composable get() = LocalColors.current
    val design: PuddingDesign @Composable get() = LocalDesign.current
}

@Composable
fun PuddingTheme(
    dark: Boolean = isSystemInDarkTheme(),
    colors: PuddingColors = if (dark) PuddingColors.Dark else PuddingColors.Light,
    design: PuddingDesign = PuddingDesign(),
    content: @Composable () -> Unit,
) {
    val scheme = if (dark) darkColorScheme() else lightColorScheme()
    val font = design.fontSize
    val typography = Typography(
        bodyLarge = TextStyle(fontSize = font.sp, lineHeight = (font * 1.5f).sp),
        bodyMedium = TextStyle(fontSize = (font - 2).sp, lineHeight = (font * 1.4f).sp),
        bodySmall = TextStyle(fontSize = (font - 4).coerceAtLeast(10f).sp, lineHeight = (font * 1.25f).sp),
        titleLarge = TextStyle(fontSize = (font + 6).sp, lineHeight = (font * 1.8f).sp, fontWeight = FontWeight.SemiBold),
        titleMedium = TextStyle(fontSize = font.sp, lineHeight = (font * 1.5f).sp, fontWeight = FontWeight.SemiBold),
        labelLarge = TextStyle(fontSize = (font - 2).sp, lineHeight = (font * 1.3f).sp, fontWeight = FontWeight.Medium),
        labelMedium = TextStyle(fontSize = (font - 3).sp, lineHeight = (font * 1.2f).sp, fontWeight = FontWeight.Medium),
    )
    CompositionLocalProvider(LocalColors provides colors, LocalDesign provides design) {
        MaterialTheme(
            colorScheme = scheme.copy(primary = colors.solid, onPrimary = colors.inverse, primaryContainer = colors.selected, onPrimaryContainer = colors.text, secondary = colors.secondary, onSecondary = colors.inverse, surface = colors.surface, onSurface = colors.text, surfaceVariant = colors.subtle, onSurfaceVariant = colors.secondary, outline = colors.border, outlineVariant = colors.border, error = colors.error, surfaceTint = Color.Transparent),
            typography = typography,
            shapes = Shapes(extraSmall = RoundedCornerShape((design.radius / 2).dp), small = RoundedCornerShape((design.radius * .75f).dp), medium = RoundedCornerShape(design.radius.dp), large = RoundedCornerShape((design.radius * 1.5f).dp), extraLarge = RoundedCornerShape((design.radius * 2).dp)),
            content = content,
        )
    }
}
