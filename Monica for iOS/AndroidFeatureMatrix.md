# Monica Android -> iOS 功能对齐矩阵

本矩阵是 iOS 一次性对齐 Android 端的验收控制面。每个 Android 功能必须在 iOS 中标记为：

- `已实现`：iOS 已有可用实现，并通过自动化或人工验收。
- `开发中`：已有部分代码或接口，仍缺完整体验/测试/真机验证。
- `待实现`：尚未开始或只有占位设计。
- `iOS 原生替代`：Android 平台专属能力无法原样复制，使用 iOS 原生机制完成同等用户目标。
- `平台不可用`：iOS 不允许实现，必须在产品文案中解释。

## P0 基线

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| MDBX 本地保险库 | `mdbx/`, `Monica for Android/app/src/main/java/takagi/ru/monica/mdbx` | SwiftUI + UniFFI + `MonicaMDBX`/`MonicaStorage` | 已实现 | 可创建、打开、锁定、重开 MDBX；Rust smoke、SwiftPM、XCTest 通过 |
| Android -> iOS 功能矩阵 | Android routes/services/docs | 本文档持续更新 | 已实现 | 每个里程碑更新状态和验收结果 |
| 仓库基线提交 | Git workspace | 推送 iOS/MDBX 源码到 `zzstar101/Monica` | 已实现 | 已推送 `e97ca360`；源码优先，`.build`、`Build`、`target` 等产物排除 |

## 核心保险库与条目

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| 密码条目 | `PasswordViewModel`, `PasswordListScreen`, `AddEditPasswordScreen` | 一等 `login` 条目 | 已实现 | 新增、编辑、搜索、收藏、软删除、恢复、AutoFill 索引同步 |
| TOTP/验证器 | `TotpViewModel`, `TotpListScreen`, `AddEditTotpScreen`, `QrScannerScreen` | 一等 `totp` 条目 | 已实现 | URI/二维码导入、验证码生成、剩余秒刷新、编辑、删除恢复 |
| 安全笔记 | `NoteViewModel`, `NoteListScreen`, `AddEditNoteScreen` | 一等 `note` 条目 | 已实现 | 新增、编辑、搜索、收藏、删除恢复 |
| 银行卡 | `BankCardViewModel`, `CardWalletScreen`, `AddEditBankCardScreen` | 一等 `card` 条目 | 已实现 | 卡号/CVV 加密 payload；列表只显示摘要；删除恢复 |
| 证件/身份 | `DocumentViewModel`, `AddEditDocumentScreen` | 一等 `identity` 条目 | 已实现 | 证件号加密 payload；列表摘要不泄漏完整敏感字段 |
| Passkey 元数据 | `PasskeyViewModel`, `PasskeyListScreen`, `PasskeyDetailScreen` | 一等 `passkey` 条目 + 系统 Passkey | 开发中 | 元数据 CRUD 可用；系统创建/认证需 AuthenticationServices 真机验收 |
| SSH Key | `AddEditSshKeyScreen`, `SshKeyDetailScreen` | 一等 `sshKey` 条目 | 已实现 | 列表、详情编辑、新增、搜索、收藏、软删除、恢复通过 XCTest；私钥以引用字段保存 |
| API Token | Android 扩展条目/自定义字段 | 一等 `apiToken` 条目 | 已实现 | 列表、详情编辑、新增、搜索、收藏、软删除、恢复通过 XCTest；Token 不在列表明文展示 |
| Wi-Fi | `AddEditWifiScreen`, `WifiDetailScreen` | 一等 `wifi` 条目 | 开发中 | 基础 CRUD、搜索、收藏、删除恢复已实现；二维码/系统分享策略待后续补齐 |
| Bitwarden Send | `SendScreen`, `AddEditSendScreen` | 一等 `send` 条目 + Bitwarden Send 同步 | 开发中 | 基础 CRUD、搜索、收藏、删除恢复已实现；Bitwarden 同步、附件支持待 P3 |
| 附件引用 | `attachments/` | 一等 `attachmentRef` + 内容存储 | 开发中 | 元数据接口已存在；需要文件内容加密、预览、迁移和同步 |

## 自动填充、Passkey 与 iOS 原生替代

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| Android AutofillService | `autofill_ng/MonicaAutofillServiceNg` | `MonicaAutoFillExtension` Credential Provider | 开发中 | QuickType identity 展示、域名匹配、搜索、解锁、填充真机通过 |
| 保存新密码 | `AutofillSaveActivity`, `AutofillSaveTransparentActivity` | Credential Provider/主 App 保存流 | 待实现 | 从系统保存请求创建或更新登录条目 |
| Inline suggestion | `autofill_inline_*` layouts | iOS QuickType/credential identity | iOS 原生替代 | 系统建议栏展示匹配账号 |
| 手动填充 Tile | `AutofillTileService` | Shortcuts/App Intents + Share/Action Extension | iOS 原生替代 | 可从快捷入口搜索并复制/打开对应条目 |
| IME 键盘填充 | `ime/MonicaInputMethodService` | 不复制；用 AutoFill、Shortcuts、Share Extension 替代 | iOS 原生替代 | 文档说明限制；关键用户路径有替代入口 |
| Accessibility 辅助填充 | `MonicaAccessibilityService` | 不复制；用 iOS 原生 AutoFill 替代 | iOS 原生替代 | 文档说明限制；无私有 API |
| Android Credential Provider Passkey | `passkey/MonicaCredentialProviderService` | AuthenticationServices Passkey | 待实现 | 支持注册、认证、RP ID 校验、associated domains |
| TOTP 常驻通知 | `AutofillOtpNotificationService`, `NotificationValidatorService` | Widget/Live Activity/短时通知安全替代 | iOS 原生替代 | 不常驻暴露敏感验证码；用户主动开启后可快速查看 |

## 同步、导入导出与外部格式

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| WebDAV 备份/恢复 | `webdav/`, `WebDavBackupScreen`, `SyncBackupScreen` | `MonicaSync` WebDAV | 已实现 | 上传、下载、SHA-256 校验、恢复前打开验证 |
| OneDrive | `OneDriveBackupScreen`, MSAL config | `CloudFileProvider` OneDrive adapter | 待实现 | 登录、浏览、创建/打开、备份、恢复 |
| Google Drive | KeePass Google Drive browser | `CloudFileProvider` Google Drive adapter | 待实现 | 登录、浏览、创建/打开、备份、恢复 |
| Android 备份包 | `DataExportImportViewModel.exportZipBackup/importZipBackup`, `WebDavHelper.createBackupZip/restoreFromBackupFile` | `AndroidBackupCodec` + App 迁移入口 | 开发中 | Storage 层已支持 Android ZIP 新版 `folders/<分类>/passwords/authenticators/bank_cards/documents/notes/passkeys/*.json` 核心条目导入，并可导出 Android 风格 ZIP；已兼容旧版 ZIP 内 `*_password.csv`、`*_totp.csv`、`*_cards_docs.csv`、`*_notes.csv` 兜底导入；已解析 `attachments/attachments_meta.json` 附件 manifest 元数据和 blob 路径；设置页已接入 `.zip` 文件选择、预览后确认导入当前 vault、导出 Android 风格 ZIP；加密备份、附件内容落盘/预览/迁移、回收站/配置恢复待接入 |
| CSV 导入导出 | import/export screens | CSV importer/exporter | 已实现 | Storage 层 CSV 编解码、字段映射、错误报告脱敏已实现；设置页已接入 iOS 文件导入/导出、预览后确认导入当前项目 |
| KDBX/KeePass | `LocalKeePass*`, `keepass/` | KDBX 读写兼容 | 待实现 | 打开、编辑、保存、回收站、附件、云文件源 |
| Bitwarden 同步 | `bitwarden/`, `SyncQueue` | Bitwarden 双向同步 | 待实现 | 登录、vault/folder、密码、TOTP、Send、附件、冲突处理 |

## 管理、设置与商业化

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| 分类/快速筛选 | `CategoryDao`, `PasswordQuickFilter*` | iOS 分类、筛选、批量管理 | 待实现 | 分类 CRUD、筛选、批量移动/删除 |
| 堆叠分组 | `PasswordGrouping`, `StackedPasswordGroup` | iOS 分组/堆叠视图 | 待实现 | 按备注/网站/应用/标题等策略分组 |
| 字段/页面定制 | `PageAdjustmentCustomizationScreen`, `PasswordFieldCustomizationScreen` | iOS 风格显示偏好 | 待实现 | 字段显示、卡片密度、底部导航偏好可配置 |
| 主题/图标 | `ColorSchemeSelectionScreen`, `IconSettingsScreen` | iOS 风格主题和图标设置 | 待实现 | 深浅色、强调色、图标显示策略、App Icon 变体 |
| 安全分析 | `SecurityAnalysisScreen` | 安全中心 | 待实现 | 弱密码、复用、泄露风险、修复建议 |
| 重复项清理 | `DedupEngineScreen` | 安全中心重复项合并 | 待实现 | 重复检测、忽略列表、合并预览、可撤销 |
| 密码历史/时间线 | `TimelineScreen`, `PasswordHistoryDao`, `OperationLogDao` | 历史版本和操作时间线 | 待实现 | 条目历史、版本恢复、操作审计 |
| Plus/支付 | `plus/`, `MonicaPlusScreen`, `PaymentScreen` | StoreKit 2 + 现有 Plus 映射 | 待实现 | Apple IAP 购买/恢复；Plus/CDK 与 IAP 权益统一 |
| 权限管理 | `PermissionManagementScreen` | iOS 权限状态中心 | 待实现 | 相机、AutoFill、通知、App Group/Keychain 状态可见 |
| 开发者设置 | `DeveloperSettingsScreen` | iOS debug/diagnostics | 待实现 | 安全脱敏诊断、同步日志、fixture 导入 |

## 扩展与设备范围

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| 快捷入口 | Quick Tile, launcher aliases | Shortcuts/App Intents | 待实现 | 可搜索、打开、复制用户选择的条目 |
| 分享/导入 | Android intents/file picker | Share/Action Extension | 待实现 | 从文件、URL、二维码、文本导入到当前 vault |
| 小组件 | Android 通知/快捷状态 | iOS Widget | 待实现 | 安全显示 TOTP/快捷状态，不泄漏秘密 |
| iPad | Android 大屏适配 | iPhone 优先 + iPad 自适应 | 开发中 | iPad 不崩溃、不遮挡；后续再做一等分栏体验 |
| Apple Watch | 无直接 Android 等价 | 后置 | 待实现 | 本轮不作为完成条件 |
