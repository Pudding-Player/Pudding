package dev.pudding.android

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import dev.pudding.ui.*
import java.time.LocalDate
import java.time.LocalTime

class MainActivity : ComponentActivity() {
    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        setContent { PuddingApp() }
    }
}

@Composable
private fun PuddingApp() {
    val systemDark = isSystemInDarkTheme()
    var dark by rememberSaveable { mutableStateOf(systemDark) }
    var page by rememberSaveable { mutableStateOf("controls") }
    var checked by rememberSaveable { mutableStateOf(true) }
    var name by rememberSaveable { mutableStateOf("") }
    var selection by rememberSaveable { mutableStateOf("default") }
    var progress by rememberSaveable { mutableFloatStateOf(40f) }
    var running by rememberSaveable { mutableStateOf(false) }
    var message by rememberSaveable { mutableStateOf("") }
    var dialog by remember { mutableStateOf(false) }
    var notice by remember { mutableStateOf(false) }
    var quantity by rememberSaveable { mutableIntStateOf(2) }
    var chips by remember { mutableStateOf(setOf("default")) }
    var bounds by remember { mutableStateOf(20f..80f) }
    var mobilePage by rememberSaveable { mutableStateOf("home") }
    var expanded by rememberSaveable { mutableStateOf(true) }
    var selectedFile by rememberSaveable { mutableStateOf(false) }
    var sheet by remember { mutableStateOf(false) }
    var dateOpen by remember { mutableStateOf(false) }
    var timeOpen by remember { mutableStateOf(false) }
    var date by rememberSaveable { mutableStateOf("2026-10-16") }
    var time by rememberSaveable { mutableStateOf("09:30") }
    PuddingTheme(dark) {
        Surface(Modifier.fillMaxSize(), color = PuddingTheme.colors.surface) {
            Column(Modifier.fillMaxSize().safeDrawingPadding().padding(horizontal = 24.dp)) {
                Row(Modifier.fillMaxWidth().padding(vertical = 20.dp), verticalAlignment = Alignment.CenterVertically) {
                    Column(Modifier.weight(1f)) { Text("Pudding", style = MaterialTheme.typography.titleLarge); Text("组件与主题", style = MaterialTheme.typography.bodySmall, color = PuddingTheme.colors.muted) }
                    PuddingSwitch(dark, { dark = it }, "深色主题")
                }
                PuddingTabs(page, { page = it }, listOf(SelectOption("controls", "控件"), SelectOption("composites", "组合"), SelectOption("mobile", "移动端"), SelectOption("settings", "设置")), Modifier.fillMaxWidth())
                Column(Modifier.weight(1f).verticalScroll(rememberScrollState()).padding(vertical = 24.dp), verticalArrangement = Arrangement.spacedBy(24.dp)) {
                    when (page) {
                        "controls" -> {
                            PuddingSection("操作") {
                                Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                    PuddingButton("主要操作", { notice = !notice })
                                    PuddingButton("次要操作", { dialog = true }, variant = ButtonVariant.Secondary)
                                }
                                if (notice) PuddingNotice("操作已完成", actionLabel = "关闭", onAction = { notice = false })
                            }
                            PuddingSection("输入与选择") {
                                PuddingTextField(name, { name = it }, "配置名称", Modifier.fillMaxWidth(), placeholder = "输入名称")
                                PuddingSelect(selection, { selection = it }, "配置", listOf(SelectOption("default", "默认配置"), SelectOption("custom", "自定义配置")), Modifier.fillMaxWidth())
                                PuddingCheckbox(checked, { checked = it }, "保存为默认配置")
                                PuddingSlider(progress, { progress = it }, Modifier.fillMaxWidth())
                                PuddingProgress(progress / 100f)
                            }
                        }
                        "composites" -> {
                            PuddingTaskCard("采集路线", "使用保存的路线与配置", if (running) "运行中" else "已就绪", { running = !running }, running = running, pinned = true)
                            PuddingListItem("本地配置", "管理保存的配置", { dialog = true }, leading = { PuddingAvatar("木偶") }, trailing = { PuddingBadge("已保存") })
                            PuddingMessageBubble("开始一项新任务。")
                            PuddingMessageBubble("整理我的采集配置。", user = true)
                            PuddingComposer(message, { message = it }, { message = ""; notice = true })
                            if (notice) PuddingNotice("输入已提交", actionLabel = "关闭", onAction = { notice = false })
                        }
                        "mobile" -> {
                            PuddingSection("移动导航") {
                                PuddingTopAppBar("任务管理", onBack = { notice = true }, onAction = { sheet = true })
                                PuddingBottomNavigation(mobilePage, { mobilePage = it }, listOf(PuddingNavigationItem("home", "首页", PuddingSymbol.Home), PuddingNavigationItem("tasks", "任务", PuddingSymbol.Tasks, 3), PuddingNavigationItem("settings", "设置", PuddingSymbol.Settings)))
                                PuddingStepIndicator(1, listOf("选择文件", "调整配置", "完成"))
                                PuddingFloatingActionButton("新建任务", { sheet = true }, extended = true)
                            }
                            PuddingSection("选择控件") {
                                val choices = listOf(SelectOption("default", "默认配置"), SelectOption("custom", "自定义配置"))
                                PuddingRadioGroup(selection, { selection = it }, choices)
                                PuddingFilterChips(chips, { chips = it }, choices)
                                PuddingStepper(quantity, { quantity = it })
                                PuddingRangeSlider(bounds, { bounds = it }, Modifier.fillMaxWidth())
                                PuddingButton("日期 $date", { dateOpen = true }, variant = ButtonVariant.Secondary)
                                PuddingButton("时间 $time", { timeOpen = true }, variant = ButtonVariant.Secondary)
                            }
                            PuddingSection("卡片与列表") {
                                PuddingStatCard("本月完成", "128", Modifier.fillMaxWidth(), "比上月增加 12 项")
                                PuddingProfileCard("木偶", "本地工作空间", { notice = true })
                                PuddingFileItem("采集路线.json", "24 KB · 刚刚更新", { notice = true }, selected = selectedFile, onSelectedChange = { selectedFile = it })
                                PuddingUploadCard("任务附件.zip", .45f, { notice = true })
                            }
                            PuddingSection("提示与布局") {
                                PuddingSkeleton()
                                PuddingCircularProgress(.6f)
                                PuddingDivider(label = "高级选项")
                                PuddingAccordion("高级设置", expanded, { expanded = it }) { Text("这些内容直接来自同一份组件库。") }
                                if (notice) PuddingSnackbar("操作已完成", actionLabel = "撤销", onAction = { notice = false }, onDismiss = { notice = false })
                            }
                        }
                        else -> {
                            PuddingSection("外观") {
                                PuddingSettingRow("深色主题", description = "调整应用的明暗外观") { PuddingSwitch(dark, { dark = it }, "深色主题") }
                                PuddingSettingRow("启用扩展", description = "组件状态随应用保留") { PuddingSwitch(checked, { checked = it }, "启用扩展") }
                            }
                            PuddingSection("关于") { Text("Pudding 0.1.0", style = MaterialTheme.typography.bodyMedium); Text("原生 Android · Jetpack Compose", style = MaterialTheme.typography.bodySmall, color = PuddingTheme.colors.muted) }
                        }
                    }
                }
            }
            if (dialog) PuddingDialog("确认操作", "调整后的配置将在这里确认。", { dialog = false }, { dialog = false; notice = true })
            PuddingBottomSheet(sheet, { sheet = false }, "新建任务") { Text("选择配置后继续。这里可以放置真实业务表单。"); PuddingButton("完成", { sheet = false; notice = true }, Modifier.fillMaxWidth()) }
            PuddingDatePicker(dateOpen, LocalDate.parse(date), { dateOpen = false }, { date = it.toString() })
            PuddingTimePicker(timeOpen, LocalTime.parse(time), { timeOpen = false }, { time = it.toString() })
        }
    }
}
