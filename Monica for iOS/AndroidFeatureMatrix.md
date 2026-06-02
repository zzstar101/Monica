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
| MDBX 本地保险库 | `mdbx/`, `Monica for Android/app/src/main/java/takagi/ru/monica/mdbx` | SwiftUI + 上游通用 `mdbx-ffi` UniFFI + `MonicaMDBX`/`MonicaStorage` | 已实现 | 已从旧 `mdbx-ios-ffi` 迁移到通用 FFI；iOS typed 业务 API 在 wrapper 内映射为 generic entry payload JSON；可创建、打开、锁定、重开 MDBX；Rust smoke、SwiftPM、XCTest 通过 |
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
| Wi-Fi | `AddEditWifiScreen`, `WifiDetailScreen` | 一等 `wifi` 条目 | 已实现 | 基础 CRUD、搜索、收藏、删除恢复已实现；已生成标准 `WIFI:T:...;S:...;P:...;H:...;;` QR payload，支持 WPA/WEP/open 归一、隐藏网络标记和特殊字符转义；编辑页已渲染 192x192 二维码图片，显示二维码内容并接入 iOS ShareLink；系统 Wi-Fi 配置写入属于 iOS 平台限制，不作为本轮对齐要求 |
| Bitwarden Send | `SendScreen`, `AddEditSendScreen` | 一等 `send` 条目 + Bitwarden Send 同步 | 开发中 | 基础 CRUD、搜索、收藏、删除恢复已实现；Bitwarden 同步、附件支持待 P3 |
| 附件引用 | `attachments/` | 一等 `attachmentRef` + 内容存储 | 开发中 | 元数据接口已存在；Android 备份确认导入时已可落库附件元数据并 remap 父密码条目；Android 备份 `.enc` 密文 blob 已保存到通用本地附件内容仓库，metadata 保留 source/downloadState/wrapped CEK/localPath；iOS Vault 页已显示附件引用列表，支持按文件名/类型/状态/source/hash/localPath/关联条目 ID 搜索，并支持软删除与恢复；App 会话已可基于 metadata 检查本地附件密文是否存在并读取密文 bytes，状态文案不泄漏 hash、wrapped key、本地路径或密文内容；Storage 已支持 Android 本地附件格式 `12B IV + ciphertext + 16B tag` 的 AES-256-GCM raw CEK 解密，App 会话已可将解密内容物化为清洗文件名的临时预览文件且不把 hash/wrapped key/localPath/明文写入状态文案；Vault 附件卡片已接入 iOS QuickLook 预览按钮，App 层通过可注入 CEK provider 准备预览 URL 并在关闭时清理临时文件，无 provider 时返回脱敏“内容密钥不可用”提示；Android wrapped CEK 解包、迁移和同步仍待后续 |

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
| Android 备份包 | `DataExportImportViewModel.exportZipBackup/importZipBackup`, `WebDavHelper.createBackupZip/restoreFromBackupFile` | `AndroidBackupCodec` + App 迁移入口 | 开发中 | Storage 层已支持 Android ZIP 新版 `folders/<分类>/passwords/authenticators/bank_cards/documents/notes/passkeys/*.json` 核心条目导入，并可导出 Android 风格 ZIP；已兼容旧版 ZIP 内 `*_password.csv`、`*_totp.csv`、`*_cards_docs.csv`、`*_notes.csv` 兜底导入；已解析 `attachments/attachments_meta.json` 附件 manifest 元数据、blob 路径和 `attachments/*.enc` 密文 bytes，并在 App 预览文案中显示附件数量；确认导入时已把附件元数据写入 MDBX metadata repository，`parentPasswordId` 会 remap 到新建 iOS login entry id，附件密文保存到通用本地附件内容仓库且 metadata 记录 source/downloadState/wrapped CEK/localPath；导入后的附件引用已进入 Vault 页一等列表，支持搜索、软删除和恢复，并可检查/读取本地密文 blob；已补 Android 本地附件 AES-256-GCM raw CEK 解密、临时预览文件物化与 QuickLook 预览入口；已识别 Android `MONICA_ENC_V1`/`.enc.zip` 加密备份，无密码时进入设置页密码输入提示，有密码时按 Android AES-256-GCM + PBKDF2-HMAC-SHA256(100000) 格式解密后复用 ZIP 导入路径，密码错误或文件损坏会返回明确失败并允许重试；设置页已接入 `.zip` 文件选择、加密备份密码提示、预览后确认导入当前 vault、导出 Android 风格 ZIP；Android wrapped CEK 解包、迁移/同步、回收站/配置恢复待接入 |
| CSV 导入导出 | import/export screens | CSV importer/exporter | 已实现 | Storage 层 CSV 编解码、字段映射、错误报告脱敏已实现；设置页已接入 iOS 文件导入/导出、预览后确认导入当前项目 |
| KDBX/KeePass | `LocalKeePass*`, `keepass/` | KDBX 读写兼容 | 开发中 | Storage 层已按 Android `KeePassFormatInspector` 口径识别 KDBX、旧版 KDB 和未知格式；旧 KDB 会提示需先在 KeePassDX/KeePassXC 另存为 `.kdbx`；KDBX 预览会解析公开文件头中的 KDBX 3/4 版本摘要，进入等待密码/密钥文件解锁状态；设置页已接入 KDBX 文件检查、数据库密码输入、密钥文件选择和解锁输入预检，预检只保留脱敏凭据摘要且不会写入当前 MDBX vault；Storage 已新增可注入 `KeePassDatabaseReader`、脱敏 `KeePassUnlockCredentials` 和只读分组/条目 snapshot 模型，App 层可用 fake reader 预览只读树结构并保持 MDBX vault 不变；Storage/App 已新增只读导入计划预览，可把 snapshot 中活动条目和回收站条目都归入可预览候选，并用 `isDeleted` 标记回收站候选，设置页显示脱敏候选/跳过摘要；导入计划会统计待解码的密码字段、TOTP 和附件数量，设置页与确认导入结果会显示这些 pending 能力且不泄漏数据库密码或 key file 内容；App 已支持确认导入只读计划中的登录条目元数据，写入 title/username/url，密码固定留空并以状态文案说明秘密字段待 KDBX 解码器接入，导入时会把 KeePass 分组路径映射到 iOS 分类/项目（如 `/Work/Clients` -> `KeePass / Work / Clients`），并保留本次 KeePass entry/group 原生 ID 到 iOS login/category 的内部脱敏映射；带 `hasTotp` 的 KeePass 候选会在同一分类创建 iOS TOTP 占位条目，secret 固定为空，issuer/account 使用可用元数据，并把 KeePass entry UUID 映射到 iOS login 与 TOTP 条目 ID；带附件摘要的 KeePass 候选会创建 iOS `attachmentRef` 占位元数据，关联到新建 login，记录文件名、媒体类型、原始大小和 content hash，`storageMode=keepass-kdbx-placeholder`、`downloadState=pending-kdbx-decode`，并把 iOS 附件 ID 写入本次导入引用映射；回收站候选会创建对应 login/TOTP/附件元数据后立即通过 iOS repository 软删除，`AppKeePassImportedEntryReference.importedAsDeleted` 记录源 KeePass 条目到 iOS 已删除元数据的映射；导入后切到首个导入分类并清理 KDBX 文件、密码和 key file 临时状态；完整 KDBX 解码打开、秘密字段导入、TOTP secret 导入、附件内容导入/解密、编辑、保存、云文件源和 KeePass 原生回收站还原语义仍待后续 |
| Bitwarden 同步 | `bitwarden/`, `SyncQueue` | Bitwarden 双向同步 | 待实现 | 登录、vault/folder、密码、TOTP、Send、附件、冲突处理 |

## 管理、设置与商业化

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| 分类/快速筛选 | `CategoryDao`, `PasswordQuickFilter*` | iOS 分类、筛选、批量管理 | 开发中 | Vault 页已接入 iOS 横向快速筛选条，支持“全部 / 当前分类 / 收藏 / 回收站”入口，显示当前分类活动条目、收藏条目和已删除条目计数；点击收藏会清空搜索并联动所有一等条目的收藏筛选，点击回收站会隐藏活动列表并保留现有删除/恢复区块，锁库时筛选状态复位；Vault 页已接入分类管理条，支持分类列表、创建、切换、重命名和删除空分类，创建后自动切到新分类，切换会刷新全部条目列表并隔离搜索、收藏筛选和批量选择状态，含活动或回收站条目的分类会拒绝删除；Vault 页已接入批量管理条，支持当前可见结果的全选、选择态、批量软删除、回收站中批量恢复，以及跨分类批量移动；批量移动覆盖 `login/note/totp/card/identity/passkey/sshKey/apiToken/wifi/send/attachmentRef`，会从源分类移除、写入目标分类、保留原条目 ID，移动后刷新当前分类、清空搜索和选择态，并写入脱敏 `.moved` 时间线；当前分类列表/重命名/删除由 iOS Storage 会话索引维护，Rust UniFFI/MDBX 原生 project list/rename/delete 持久化仍待后续 |
| 堆叠分组 | `PasswordGrouping`, `StackedPasswordGroup` | iOS 分组/堆叠视图 | 开发中 | 密码页已接入“堆叠分组”开关，支持按网站 host 聚合并把 `www.` 归一到主域名；无 URL 条目回退标题分组；分组摘要显示条目数、标题预览和账号摘要，不包含密码；搜索、收藏和回收站筛选会联动分组结果，锁库时分组模式复位；备注/应用/自定义策略仍待后续 |
| 字段/页面定制 | `PageAdjustmentCustomizationScreen`, `PasswordFieldCustomizationScreen` | iOS 风格显示偏好 | 开发中 | 设置页已接入 iOS 风格显示偏好，支持密码列表账号字段显示/隐藏、网址字段显示/隐藏、舒适/紧凑卡片密度和底部导航文字显示/隐藏；偏好通过 UserDefaults 持久化，锁库不重置；完整字段顺序、页面重排和更多条目类型显示策略仍待后续 |
| 主题/图标 | `ColorSchemeSelectionScreen`, `IconSettingsScreen` | iOS 风格主题和图标设置 | 开发中 | 设置页已接入 iOS 外观偏好，支持跟随系统/浅色/深色颜色模式、Monica/绿色/蓝色/橙色强调色和密码列表图标彩色/单色/隐藏策略；偏好通过 UserDefaults 持久化，锁库不重置；App 根视图会套用 `preferredColorScheme` 和系统 tint，密码列表及堆叠分组会按图标策略显示；App Icon 变体和真机资产切换仍待后续 |
| 安全分析 | `SecurityAnalysisScreen` | 安全中心 | 开发中 | 设置页已显示安全中心第一版，统计弱密码、复用密码、泄露风险和重复登录条目数且不泄漏具体密码；泄露风险第一版使用本地 SHA-256 指纹库命中统计；已显示弱密码、复用、泄露风险和重复项的修复建议列表；在线泄露库同步、历史版本和自动修复动作待后续 |
| 重复项清理 | `DedupEngineScreen` | 安全中心重复项合并 | 开发中 | 已在安全中心显示重复登录条目摘要和合并预览，按标题/用户名/URL 去空白小写后分组；支持合并预览后软删除重复项并可从回收站恢复；支持忽略重复组并从摘要/预览中隐藏，设置页可恢复已忽略重复项；支持最近一次重复项合并的专门撤销入口；会话内登录 CRUD 操作时间线已接入安全中心；完整跨会话操作历史和版本恢复待后续 |
| 密码历史/时间线 | `TimelineScreen`, `PasswordHistoryDao`, `OperationLogDao` | 历史版本和操作时间线 | 开发中 | App 会话内已记录 `login/note/totp/card/identity/passkey/sshKey/apiToken/wifi/send` 条目创建、更新、删除、恢复操作时间线，并记录 `attachmentRef` 删除、恢复和 QuickLook 预览准备操作时间线；事件只包含动作、类型、条目 ID、标题和时间，不包含密码、用户名、笔记正文、TOTP seed、卡号/CVV、证件号、Passkey/SSH 私钥引用、API token、Wi-Fi 密码、Send 内容、附件 hash、wrapped key、本地密文路径、密文内容或附件明文等秘密；持久化历史版本、跨会话审计和版本恢复待后续 |
| Plus/支付 | `plus/`, `MonicaPlusScreen`, `PaymentScreen` | StoreKit 2 + 现有 Plus 映射 | 待实现 | Apple IAP 购买/恢复；Plus/CDK 与 IAP 权益统一 |
| 权限管理 | `PermissionManagementScreen` | iOS 权限状态中心 | 开发中 | 设置页已显示相机、AutoFill、通知、App Group、Keychain 状态中心；相机读取系统授权状态，通知读取 `UNUserNotificationCenter` 授权状态，AutoFill/App Group/Keychain 读取当前 App 配置状态；相机/通知提供 iOS App 设置入口；签名真机 entitlement 校验待后续 |
| 开发者设置 | `DeveloperSettingsScreen` | iOS debug/diagnostics | 开发中 | 设置页已显示脱敏诊断中心，覆盖主存储、MDBX 桥接、App Group、本机标识脱敏值、AutoFill 索引状态和 WebDAV 同步状态摘要；详细同步日志、fixture 导入和导出诊断包待后续 |

## 扩展与设备范围

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| 快捷入口 | Quick Tile, launcher aliases | Shortcuts/App Intents | 待实现 | 可搜索、打开、复制用户选择的条目 |
| 分享/导入 | Android intents/file picker | Share/Action Extension | 待实现 | 从文件、URL、二维码、文本导入到当前 vault |
| 小组件 | Android 通知/快捷状态 | iOS Widget | 待实现 | 安全显示 TOTP/快捷状态，不泄漏秘密 |
| iPad | Android 大屏适配 | iPhone 优先 + iPad 自适应 | 开发中 | iPad 不崩溃、不遮挡；后续再做一等分栏体验 |
| Apple Watch | 无直接 Android 等价 | 后置 | 待实现 | 本轮不作为完成条件 |
