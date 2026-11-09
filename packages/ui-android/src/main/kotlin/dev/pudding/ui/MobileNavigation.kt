@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package dev.pudding.ui

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.semantics.contentDescription
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

enum class PuddingSymbol { Back, Add, Minus, More, Home, Tasks, Settings, File, Upload, Check, Chevron, Close }

/** Small shared vector set, independent of fonts and platform emoji rendering. */
@Composable
fun PuddingGlyph(symbol: PuddingSymbol, modifier: Modifier = Modifier) {
    val color = LocalContentColor.current
    Canvas(modifier.size(24.dp)) {
        fun point(x: Float, y: Float) = Offset(size.width * x / 24f, size.height * y / 24f)
        val width = 1.8.dp.toPx()
        fun line(x: Float, y: Float, xx: Float, yy: Float) = drawLine(color, point(x, y), point(xx, yy), width, StrokeCap.Round)
        when (symbol) {
            PuddingSymbol.Back -> { line(15f, 5f, 8f, 12f); line(8f, 12f, 15f, 19f) }
            PuddingSymbol.Chevron -> { line(9f, 5f, 16f, 12f); line(16f, 12f, 9f, 19f) }
            PuddingSymbol.Add -> { line(5f, 12f, 19f, 12f); line(12f, 5f, 12f, 19f) }
            PuddingSymbol.Minus -> line(5f, 12f, 19f, 12f)
            PuddingSymbol.More -> listOf(5f, 12f, 19f).forEach { drawCircle(color, 1.5.dp.toPx(), point(it, 12f)) }
            PuddingSymbol.Close -> { line(6f, 6f, 18f, 18f); line(18f, 6f, 6f, 18f) }
            PuddingSymbol.Check -> { line(4f, 12f, 9f, 17f); line(9f, 17f, 20f, 6f) }
            PuddingSymbol.Home -> { line(3f, 11f, 12f, 3f); line(12f, 3f, 21f, 11f); line(6f, 10f, 6f, 21f); line(6f, 21f, 18f, 21f); line(18f, 21f, 18f, 10f) }
            PuddingSymbol.Tasks -> { for (y in listOf(6f, 12f, 18f)) { line(9f, y, 20f, y); drawCircle(color, 1.dp.toPx(), point(4f, y)) } }
            PuddingSymbol.Settings -> { drawCircle(color, size.minDimension / 3f, point(12f, 12f), style = Stroke(width)); drawCircle(color, size.minDimension / 8f, point(12f, 12f), style = Stroke(width)) }
            PuddingSymbol.File -> { line(6f, 3f, 14f, 3f); line(14f, 3f, 19f, 8f); line(19f, 8f, 19f, 21f); line(19f, 21f, 6f, 21f); line(6f, 21f, 6f, 3f); line(14f, 3f, 14f, 8f); line(14f, 8f, 19f, 8f) }
            PuddingSymbol.Upload -> { line(12f, 16f, 12f, 3f); line(12f, 3f, 7f, 8f); line(12f, 3f, 17f, 8f); line(4f, 16f, 4f, 21f); line(4f, 21f, 20f, 21f); line(20f, 21f, 20f, 16f) }
        }
    }
}

@Composable
fun PuddingTopAppBar(title: String, modifier: Modifier = Modifier, onBack: (() -> Unit)? = null, actionLabel: String = "更多", onAction: (() -> Unit)? = null) {
    TopAppBar(title = { Text(title, maxLines = 1, overflow = TextOverflow.Ellipsis) }, modifier = modifier,
        navigationIcon = { onBack?.let { PuddingIconButton("返回", it) { PuddingGlyph(PuddingSymbol.Back) } } },
        actions = { onAction?.let { PuddingIconButton(actionLabel, it) { PuddingGlyph(PuddingSymbol.More) } } },
        windowInsets = WindowInsets(0, 0, 0, 0), colors = TopAppBarDefaults.topAppBarColors(containerColor = PuddingTheme.colors.surface))
}

data class PuddingNavigationItem(val value: String, val label: String, val symbol: PuddingSymbol = PuddingSymbol.Home, val badgeCount: Int = 0)

@Composable
fun PuddingBottomNavigation(value: String, onValueChange: (String) -> Unit, items: List<PuddingNavigationItem>, modifier: Modifier = Modifier) {
    NavigationBar(modifier, containerColor = PuddingTheme.colors.surface, tonalElevation = 0.dp, windowInsets = WindowInsets(0, 0, 0, 0)) {
        items.forEach { item ->
            NavigationBarItem(selected = value == item.value, onClick = { onValueChange(item.value) }, label = { Text(item.label, maxLines = 1) }, icon = {
                BadgedBox(badge = { if (item.badgeCount > 0) Badge { Text(if (item.badgeCount > 99) "99+" else item.badgeCount.toString()) } }) { PuddingGlyph(item.symbol) }
            }, colors = NavigationBarItemDefaults.colors(indicatorColor = PuddingTheme.colors.selected, selectedIconColor = PuddingTheme.colors.text, selectedTextColor = PuddingTheme.colors.text, unselectedIconColor = PuddingTheme.colors.muted, unselectedTextColor = PuddingTheme.colors.muted))
        }
    }
}

@Composable
fun PuddingFloatingActionButton(label: String, onClick: () -> Unit, modifier: Modifier = Modifier, extended: Boolean = false) {
    if (extended) ExtendedFloatingActionButton(onClick = onClick, icon = { PuddingGlyph(PuddingSymbol.Add) }, text = { Text(label) }, modifier = modifier, containerColor = PuddingTheme.colors.solid, contentColor = PuddingTheme.colors.inverse)
    else FloatingActionButton(onClick, modifier.semantics { contentDescription = label }, containerColor = PuddingTheme.colors.solid, contentColor = PuddingTheme.colors.inverse) { PuddingGlyph(PuddingSymbol.Add) }
}

@Composable
fun PuddingStepIndicator(current: Int, items: List<String>, modifier: Modifier = Modifier) {
    Row(modifier.fillMaxWidth(), verticalAlignment = Alignment.Top, horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        items.forEachIndexed { index, label ->
            val active = index <= current
            Column(Modifier.weight(1f), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Surface(shape = CircleShape, color = if (active) PuddingTheme.colors.solid else PuddingTheme.colors.subtle, contentColor = if (active) PuddingTheme.colors.inverse else PuddingTheme.colors.muted) {
                    Box(Modifier.size(32.dp), contentAlignment = Alignment.Center) { if (index < current) PuddingGlyph(PuddingSymbol.Check, Modifier.size(18.dp)) else Text((index + 1).toString(), style = MaterialTheme.typography.labelLarge) }
                }
                Text(label, style = MaterialTheme.typography.labelMedium, color = if (active) PuddingTheme.colors.text else PuddingTheme.colors.muted)
            }
        }
    }
}
