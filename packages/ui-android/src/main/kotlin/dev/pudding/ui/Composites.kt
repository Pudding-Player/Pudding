package dev.pudding.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp

@Composable
fun PuddingSettingRow(title: String, modifier: Modifier = Modifier, description: String? = null, control: @Composable () -> Unit) {
    Column(modifier) {
        Row(Modifier.fillMaxWidth().heightIn(min = 64.dp).padding(vertical = 12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
            Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                Text(title, style = MaterialTheme.typography.bodyLarge)
                if (!description.isNullOrBlank()) Text(description, style = MaterialTheme.typography.bodySmall, color = PuddingTheme.colors.muted)
            }
            control()
        }
        HorizontalDivider(thickness = PuddingTheme.design.borderWidth.dp, color = PuddingTheme.colors.border)
    }
}

@Composable
fun PuddingSection(title: String, modifier: Modifier = Modifier, description: String? = null, content: @Composable ColumnScope.() -> Unit) {
    Column(modifier.fillMaxWidth(), verticalArrangement = Arrangement.spacedBy(16.dp)) {
        Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(title, style = MaterialTheme.typography.titleMedium)
            if (!description.isNullOrBlank()) Text(description, style = MaterialTheme.typography.bodyMedium, color = PuddingTheme.colors.muted)
        }
        content()
    }
}

@Composable
fun PuddingEmptyState(title: String, description: String, modifier: Modifier = Modifier, action: (@Composable () -> Unit)? = null) {
    Column(modifier.fillMaxWidth().padding(24.dp), horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(12.dp)) {
        Text("☾", style = MaterialTheme.typography.titleLarge, color = PuddingTheme.colors.accent)
        Text(title, style = MaterialTheme.typography.titleMedium)
        Text(description, style = MaterialTheme.typography.bodyMedium, color = PuddingTheme.colors.muted)
        action?.invoke()
    }
}

@Composable
fun PuddingTaskCard(title: String, description: String, state: String, onRun: () -> Unit, modifier: Modifier = Modifier, running: Boolean = false, enabled: Boolean = true, pinned: Boolean = false, onOpen: (() -> Unit)? = null) {
    OutlinedCard(onClick = { onOpen?.invoke() }, modifier = modifier.fillMaxWidth(), enabled = enabled, shape = MaterialTheme.shapes.large, colors = CardDefaults.outlinedCardColors(containerColor = if (running) PuddingTheme.colors.subtle else PuddingTheme.colors.surface), border = BorderStroke(PuddingTheme.design.borderWidth.dp, PuddingTheme.colors.border)) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(10.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically) {
                Text(title, Modifier.weight(1f), style = MaterialTheme.typography.titleMedium)
                if (pinned) Text("已置顶", color = PuddingTheme.colors.muted, style = MaterialTheme.typography.labelMedium)
            }
            Text(description, style = MaterialTheme.typography.bodyMedium, color = PuddingTheme.colors.secondary)
            Row(Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.SpaceBetween) {
                PuddingBadge(state, active = running)
                PuddingButton(if (running) "停止" else "运行", onRun, enabled = enabled)
            }
        }
    }
}

@Composable
fun PuddingMessageBubble(text: String, modifier: Modifier = Modifier, user: Boolean = false) {
    Row(modifier.fillMaxWidth(), horizontalArrangement = if (user) Arrangement.End else Arrangement.Start) {
        Surface(Modifier.widthIn(max = 520.dp), color = if (user) Color(0xFFFFD400) else PuddingTheme.colors.subtle, contentColor = if (user) Color.Black else PuddingTheme.colors.text, shape = MaterialTheme.shapes.large) { Text(text, Modifier.padding(horizontal = 16.dp, vertical = 12.dp), style = MaterialTheme.typography.bodyLarge) }
    }
}

@Composable
fun PuddingComposer(value: String, onValueChange: (String) -> Unit, onSend: () -> Unit, modifier: Modifier = Modifier, enabled: Boolean = true) {
    Surface(modifier.fillMaxWidth(), shape = MaterialTheme.shapes.large, border = BorderStroke(PuddingTheme.design.borderWidth.dp, PuddingTheme.colors.border), color = PuddingTheme.colors.surface) {
        Column(Modifier.padding(12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            TextField(value, onValueChange, Modifier.fillMaxWidth(), enabled = enabled, placeholder = { Text("告诉我你想完成什么") }, minLines = 2, maxLines = 6, colors = TextFieldDefaults.colors(focusedContainerColor = Color.Transparent, unfocusedContainerColor = Color.Transparent, focusedIndicatorColor = Color.Transparent, unfocusedIndicatorColor = Color.Transparent))
            Row(Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.End) { PuddingButton("发送", onSend, enabled = enabled && value.isNotBlank()) }
        }
    }
}

@Composable
fun PuddingDialog(title: String, text: String, onDismiss: () -> Unit, onConfirm: () -> Unit, confirmLabel: String = "确定", cancelLabel: String = "取消", busy: Boolean = false) {
    AlertDialog(onDismissRequest = { if (!busy) onDismiss() }, title = { Text(title) }, text = { Text(text) }, confirmButton = { PuddingButton(confirmLabel, onConfirm, enabled = !busy, loading = busy) }, dismissButton = { PuddingButton(cancelLabel, onDismiss, enabled = !busy, variant = ButtonVariant.Text) }, shape = MaterialTheme.shapes.extraLarge, containerColor = PuddingTheme.colors.surface)
}

@Composable
fun PuddingNotice(text: String, modifier: Modifier = Modifier, actionLabel: String? = null, onAction: () -> Unit = {}) {
    Surface(modifier.fillMaxWidth(), shape = MaterialTheme.shapes.medium, color = PuddingTheme.colors.subtle, border = BorderStroke(PuddingTheme.design.borderWidth.dp, PuddingTheme.colors.border)) {
        Row(Modifier.padding(12.dp), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
            Text(text, Modifier.weight(1f), style = MaterialTheme.typography.bodyMedium, color = PuddingTheme.colors.secondary)
            if (actionLabel != null) PuddingButton(actionLabel, onAction, variant = ButtonVariant.Text)
        }
    }
}

@Composable
fun PuddingListItem(title: String, description: String, onClick: () -> Unit, modifier: Modifier = Modifier, leading: (@Composable () -> Unit)? = null, trailing: (@Composable () -> Unit)? = null) {
    Surface(onClick = onClick, modifier = modifier.fillMaxWidth(), color = PuddingTheme.colors.surface) {
        ListItem(headlineContent = { Text(title) }, supportingContent = { Text(description) }, leadingContent = leading, trailingContent = trailing, colors = ListItemDefaults.colors(containerColor = Color.Transparent))
    }
}
