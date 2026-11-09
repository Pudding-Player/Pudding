@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class)

package dev.pudding.ui

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

data class PuddingMenuAction(val value: String, val label: String, val enabled: Boolean = true, val destructive: Boolean = false)

@Composable
fun PuddingBottomSheet(open: Boolean, onDismiss: () -> Unit, title: String, modifier: Modifier = Modifier, content: @Composable ColumnScope.() -> Unit) {
    if (!open) return
    ModalBottomSheet(onDismissRequest = onDismiss, modifier = modifier, sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true), containerColor = PuddingTheme.colors.surface, contentColor = PuddingTheme.colors.text) {
        Column(Modifier.fillMaxWidth().verticalScroll(rememberScrollState()).padding(start = 24.dp, end = 24.dp, bottom = 24.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Text(title, style = MaterialTheme.typography.titleLarge)
            content()
        }
    }
}

@Composable
fun PuddingActionSheet(open: Boolean, onDismiss: () -> Unit, title: String, actions: List<PuddingMenuAction>, onAction: (String) -> Unit) {
    PuddingBottomSheet(open, onDismiss, title) {
        actions.forEach { action ->
            TextButton(onClick = { onAction(action.value); onDismiss() }, modifier = Modifier.fillMaxWidth().heightIn(min = 48.dp), enabled = action.enabled) {
                Text(action.label, color = if (!action.enabled) PuddingTheme.colors.muted else if (action.destructive) PuddingTheme.colors.error else PuddingTheme.colors.text)
            }
        }
        PuddingButton("取消", onDismiss, Modifier.fillMaxWidth(), variant = ButtonVariant.Secondary)
    }
}

/** The anchor owns placement; expanded is controlled by the caller. */
@Composable
fun PuddingDropdownMenu(expanded: Boolean, onDismiss: () -> Unit, actions: List<PuddingMenuAction>, onAction: (String) -> Unit, modifier: Modifier = Modifier, anchor: @Composable () -> Unit) {
    Box(modifier) {
        anchor()
        DropdownMenu(expanded, onDismiss, containerColor = PuddingTheme.colors.surface) {
            actions.forEach { action ->
                DropdownMenuItem(text = { Text(action.label, color = if (!action.enabled) PuddingTheme.colors.muted else if (action.destructive) PuddingTheme.colors.error else PuddingTheme.colors.text) }, onClick = { onAction(action.value); onDismiss() }, enabled = action.enabled)
            }
        }
    }
}

/** Put this in a Scaffold snackbar host or the screen's transient-message layer. */
@Composable
fun PuddingSnackbar(text: String, modifier: Modifier = Modifier, actionLabel: String? = null, onAction: () -> Unit = {}, onDismiss: (() -> Unit)? = null) {
    Surface(modifier, shape = MaterialTheme.shapes.medium, color = PuddingTheme.colors.solid, contentColor = PuddingTheme.colors.inverse, shadowElevation = if (PuddingTheme.design.shadow) 3.dp else 0.dp) {
        Row(Modifier.fillMaxWidth().heightIn(min = 48.dp).padding(start = 16.dp, end = 4.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(text, Modifier.weight(1f).padding(vertical = 12.dp), style = MaterialTheme.typography.bodyMedium)
            if (actionLabel != null) TextButton(onAction, colors = ButtonDefaults.textButtonColors(contentColor = PuddingTheme.colors.inverse)) { Text(actionLabel) }
            if (onDismiss != null) PuddingIconButton("关闭提示", onDismiss) { PuddingGlyph(PuddingSymbol.Close) }
        }
    }
}
