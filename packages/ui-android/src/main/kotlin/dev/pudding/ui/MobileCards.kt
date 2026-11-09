package dev.pudding.ui

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.*
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp

@Composable
fun PuddingStatCard(title: String, value: String, modifier: Modifier = Modifier, supportingText: String? = null) {
    OutlinedCard(modifier, shape = MaterialTheme.shapes.large, colors = CardDefaults.outlinedCardColors(containerColor = PuddingTheme.colors.surface), border = BorderStroke(PuddingTheme.design.borderWidth.dp, PuddingTheme.colors.border)) {
        Column(Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
            Text(title, style = MaterialTheme.typography.bodyMedium, color = PuddingTheme.colors.secondary)
            Text(value, style = MaterialTheme.typography.titleLarge)
            if (!supportingText.isNullOrBlank()) Text(supportingText, style = MaterialTheme.typography.bodySmall, color = PuddingTheme.colors.muted)
        }
    }
}

@Composable
fun PuddingProfileCard(name: String, description: String, onAction: () -> Unit, modifier: Modifier = Modifier, actionLabel: String = "查看资料") {
    OutlinedCard(modifier.fillMaxWidth(), shape = MaterialTheme.shapes.large, colors = CardDefaults.outlinedCardColors(containerColor = PuddingTheme.colors.surface), border = BorderStroke(PuddingTheme.design.borderWidth.dp, PuddingTheme.colors.border)) {
        Column(Modifier.padding(20.dp), verticalArrangement = Arrangement.spacedBy(16.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(16.dp)) {
                PuddingAvatar(name)
                Column(Modifier.weight(1f), verticalArrangement = Arrangement.spacedBy(4.dp)) {
                    Text(name, style = MaterialTheme.typography.titleMedium)
                    Text(description, style = MaterialTheme.typography.bodySmall, color = PuddingTheme.colors.muted)
                }
            }
            PuddingButton(actionLabel, onAction, variant = ButtonVariant.Secondary)
        }
    }
}

@Composable
fun PuddingFileItem(name: String, metadata: String, onOpen: () -> Unit, modifier: Modifier = Modifier, selected: Boolean = false, onSelectedChange: ((Boolean) -> Unit)? = null) {
    Surface(modifier.fillMaxWidth(), color = if (selected) PuddingTheme.colors.subtle else PuddingTheme.colors.surface, shape = MaterialTheme.shapes.medium) {
        PuddingListItem(name, metadata, onOpen, leading = { PuddingGlyph(PuddingSymbol.File) }, trailing = { onSelectedChange?.let { PuddingCheckbox(selected, it, "选择$name", labelVisible = false) } })
    }
}

@Composable
fun PuddingUploadCard(name: String, progress: Float, onCancel: () -> Unit, modifier: Modifier = Modifier, error: String? = null, onRetry: () -> Unit = {}) {
    OutlinedCard(modifier.fillMaxWidth(), shape = MaterialTheme.shapes.large, colors = CardDefaults.outlinedCardColors(containerColor = PuddingTheme.colors.surface), border = BorderStroke(PuddingTheme.design.borderWidth.dp, PuddingTheme.colors.border)) {
        Column(Modifier.padding(16.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
            Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                PuddingGlyph(PuddingSymbol.Upload)
                Text(name, Modifier.weight(1f), maxLines = 2, overflow = TextOverflow.Ellipsis, style = MaterialTheme.typography.bodyMedium)
                PuddingIconButton("取消上传", onCancel) { PuddingGlyph(PuddingSymbol.Close) }
            }
            if (error.isNullOrBlank()) {
                PuddingProgress(progress, trackColor = PuddingTheme.colors.selected)
                Text(if (progress >= 1f) "上传完成" else "已上传 ${(progress.coerceIn(0f, 1f) * 100).toInt()}%", style = MaterialTheme.typography.bodySmall, color = PuddingTheme.colors.muted)
            } else PuddingNotice(error, actionLabel = "重试", onAction = onRetry)
        }
    }
}
