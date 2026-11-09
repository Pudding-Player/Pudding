@file:OptIn(androidx.compose.material3.ExperimentalMaterial3Api::class, androidx.compose.foundation.layout.ExperimentalLayoutApi::class)

package dev.pudding.ui

import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.selection.selectable
import androidx.compose.foundation.selection.selectableGroup
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalConfiguration
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.dp
import java.time.Instant
import java.time.LocalDate
import java.time.LocalTime
import java.time.ZoneOffset

@Composable
fun PuddingRadioGroup(value: String, onValueChange: (String) -> Unit, items: List<SelectOption>, modifier: Modifier = Modifier, enabled: Boolean = true) {
    Column(modifier.selectableGroup()) {
        items.forEach { item ->
            Row(Modifier.fillMaxWidth().heightIn(min = 48.dp).selectable(item.value == value, enabled = enabled, role = Role.RadioButton, onClick = { onValueChange(item.value) }).padding(horizontal = 8.dp), verticalAlignment = Alignment.CenterVertically) {
                RadioButton(item.value == value, null, enabled = enabled)
                Spacer(Modifier.width(12.dp))
                Text(item.label, color = if (enabled) PuddingTheme.colors.text else PuddingTheme.colors.muted)
            }
        }
    }
}

@Composable
fun PuddingFilterChips(value: Set<String>, onValueChange: (Set<String>) -> Unit, items: List<SelectOption>, modifier: Modifier = Modifier, enabled: Boolean = true) {
    FlowRow(modifier, horizontalArrangement = Arrangement.spacedBy(8.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
        items.forEach { item ->
            FilterChip(selected = item.value in value, onClick = { onValueChange(if (item.value in value) value - item.value else value + item.value) }, label = { Text(item.label) }, enabled = enabled,
                colors = FilterChipDefaults.filterChipColors(selectedContainerColor = PuddingTheme.colors.selected, selectedLabelColor = PuddingTheme.colors.text))
        }
    }
}

@Composable
fun PuddingStepper(value: Int, onValueChange: (Int) -> Unit, modifier: Modifier = Modifier, range: IntRange = 0..99, step: Int = 1, enabled: Boolean = true, label: String = "数量") {
    require(!range.isEmpty() && step > 0)
    val current = value.coerceIn(range)
    Row(modifier, verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(12.dp)) {
        PuddingIconButton("减少$label", { onValueChange((current.toLong() - step).coerceAtLeast(range.first.toLong()).toInt()) }, enabled = enabled && current > range.first) { PuddingGlyph(PuddingSymbol.Minus) }
        Text(current.toString(), style = MaterialTheme.typography.titleMedium)
        PuddingIconButton("增加$label", { onValueChange((current.toLong() + step).coerceAtMost(range.last.toLong()).toInt()) }, enabled = enabled && current < range.last) { PuddingGlyph(PuddingSymbol.Add) }
    }
}

@Composable
fun PuddingRangeSlider(value: ClosedFloatingPointRange<Float>, onValueChange: (ClosedFloatingPointRange<Float>) -> Unit, modifier: Modifier = Modifier, range: ClosedFloatingPointRange<Float> = 0f..100f, enabled: Boolean = true) {
    require(range.start.isFinite() && range.endInclusive.isFinite() && range.start < range.endInclusive)
    val start = value.start.coerceIn(range.start, range.endInclusive)
    val end = value.endInclusive.coerceIn(start, range.endInclusive)
    RangeSlider(value = start..end, onValueChange = onValueChange, modifier = modifier, valueRange = range, enabled = enabled,
        colors = SliderDefaults.colors(thumbColor = PuddingTheme.colors.solid, activeTrackColor = PuddingTheme.colors.solid, inactiveTrackColor = PuddingTheme.colors.selected))
}

/** Dates are calendar dates; Material's UTC epoch conversion stays inside this adapter. */
@Composable
fun PuddingDatePicker(open: Boolean, value: LocalDate?, onDismiss: () -> Unit, onConfirm: (LocalDate) -> Unit, title: String = "选择日期") {
    if (!open) return
    key(value) {
        val state = rememberDatePickerState(initialSelectedDateMillis = value?.atStartOfDay()?.toInstant(ZoneOffset.UTC)?.toEpochMilli())
        val colors = DatePickerDefaults.colors(containerColor = PuddingTheme.colors.surface, selectedDayContainerColor = PuddingTheme.colors.solid, selectedDayContentColor = PuddingTheme.colors.inverse, selectedYearContainerColor = PuddingTheme.colors.solid, selectedYearContentColor = PuddingTheme.colors.inverse, todayDateBorderColor = PuddingTheme.colors.solid, todayContentColor = PuddingTheme.colors.text)
        val maxHeight = (LocalConfiguration.current.screenHeightDp - 80).coerceAtLeast(160).dp
        DatePickerDialog(onDismissRequest = onDismiss, colors = colors, tonalElevation = 0.dp, confirmButton = { PuddingButton("确定", { state.selectedDateMillis?.let { onConfirm(Instant.ofEpochMilli(it).atOffset(ZoneOffset.UTC).toLocalDate()); onDismiss() } }, enabled = state.selectedDateMillis != null) }, dismissButton = { PuddingButton("取消", onDismiss, variant = ButtonVariant.Text) }) {
            Column(Modifier.heightIn(max = maxHeight).verticalScroll(rememberScrollState())) {
                Text(title, Modifier.padding(start = 24.dp, top = 16.dp, bottom = 8.dp), style = MaterialTheme.typography.titleMedium)
                DatePicker(state = state, title = null, headline = null, showModeToggle = false, colors = colors)
            }
        }
    }
}

@Composable
fun PuddingTimePicker(open: Boolean, value: LocalTime, onDismiss: () -> Unit, onConfirm: (LocalTime) -> Unit, title: String = "选择时间", is24Hour: Boolean = true) {
    if (!open) return
    key(value, is24Hour) {
        val state = rememberTimePickerState(initialHour = value.hour, initialMinute = value.minute, is24Hour = is24Hour)
        AlertDialog(onDismissRequest = onDismiss, title = { Text(title) }, text = { TimeInput(state, colors = TimePickerDefaults.colors(timeSelectorSelectedContainerColor = PuddingTheme.colors.selected, timeSelectorUnselectedContainerColor = PuddingTheme.colors.subtle, timeSelectorSelectedContentColor = PuddingTheme.colors.text, timeSelectorUnselectedContentColor = PuddingTheme.colors.secondary)) }, confirmButton = { PuddingButton("确定", { onConfirm(LocalTime.of(state.hour, state.minute)); onDismiss() }) }, dismissButton = { PuddingButton("取消", onDismiss, variant = ButtonVariant.Text) }, containerColor = PuddingTheme.colors.surface)
    }
}
