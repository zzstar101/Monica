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
| Bitwarden Send | `SendScreen`, `AddEditSendScreen` | 一等 `send` 条目 + Bitwarden Send 同步 | 开发中 | 基础 CRUD、搜索、收藏、删除恢复已实现；`MonicaSync` 已新增 Bitwarden pull/push 同步边界，App 会话可把本地 Send 组装为 upsert mutation 推送并预览远端 Send 摘要，状态文案不泄漏 token、远端 ID、Send 正文、notes、密码、TOTP secret 或 URL query；真实 Bitwarden 登录/API、远端 ID 映射、删除队列、附件内容同步和冲突解决 UI 仍待后续 |
| 附件引用 | `attachments/` | 一等 `attachmentRef` + 内容存储 | 开发中 | 元数据接口已存在；Android 备份确认导入时已可落库附件元数据并 remap 父密码条目；Android 备份 `.enc` 密文 blob 已保存到通用本地附件内容仓库，metadata 保留 source/downloadState/wrapped CEK/localPath；iOS Vault 页已显示附件引用列表，支持按文件名/类型/状态/source/hash/localPath/关联条目 ID 搜索，并支持软删除与恢复；App 会话已可基于 metadata 检查本地附件密文是否存在并读取密文 bytes，状态文案不泄漏 hash、wrapped key、本地路径或密文内容；Storage 已支持 Android 本地附件格式 `12B IV + ciphertext + 16B tag` 的 AES-256-GCM raw CEK 解密，App 会话已可将解密内容物化为清洗文件名的临时预览文件且不把 hash/wrapped key/localPath/明文写入状态文案；Vault 附件卡片已接入 iOS QuickLook 预览按钮，App 层通过可注入 CEK provider 准备预览 URL 并在关闭时清理临时文件，无 provider 时返回脱敏“内容密钥不可用”提示；App 会话已支持本地已下载附件内容替换第一版：使用注入 CEK 重新 AES-GCM 加密写回同一 blob 路径，更新同一附件 id 的大小、`sha256:` 内容 hash、`storageMode=ios-edited-encrypted-blob` 与 downloaded 状态，并记录不泄漏 hash/wrapped key/localPath/明文的更新时间线；Storage 已支持 Android `SecurityManager.encryptData` 的 `MDK|Base64(12B IV + ciphertext + 16B tag)` wrapped CEK 解包，App 会话可在提供 Android MDK/legacy wrapping key 时从 metadata 的 wrapped CEK 解出 raw CEK 后进入 QuickLook/写回，`V2|` Android Keystore-only 包裹仍明确不支持跨设备解包；附件迁移和同步仍待后续 |

## 自动填充、Passkey 与 iOS 原生替代

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| Android AutofillService | `autofill_ng/MonicaAutofillServiceNg` | `MonicaAutoFillExtension` Credential Provider | 开发中 | QuickType identity 展示、域名匹配、搜索、解锁、填充真机通过 |
| 保存新密码 | `AutofillSaveActivity`, `AutofillSaveTransparentActivity` | Credential Provider/主 App 保存流 | 开发中 | App 会话已新增 AutoFill 保存请求处理第一版：可把系统/扩展传入的 service identifier、username、password 和建议 title 保存为 iOS login；同一 host + username 会更新既有 login password 而不是重复创建；保存后复用现有 AutoFill 加密 index、secret snapshot 和 QuickType identity 同步链路；状态文案和时间线不泄漏 username/password/URL query；Credential Provider 真机保存回调与系统保存 UI 仍待后续 |
| Inline suggestion | `autofill_inline_*` layouts | iOS QuickType/credential identity | iOS 原生替代 | 系统建议栏展示匹配账号 |
| 手动填充 Tile | `AutofillTileService` | Shortcuts/App Intents + Share/Action Extension | iOS 原生替代 | 可从快捷入口搜索并复制/打开对应条目 |
| IME 键盘填充 | `ime/MonicaInputMethodService` | 不复制；用 AutoFill、Shortcuts、Share Extension 替代 | iOS 原生替代 | 文档说明限制；关键用户路径有替代入口 |
| Accessibility 辅助填充 | `MonicaAccessibilityService` | 不复制；用 iOS 原生 AutoFill 替代 | iOS 原生替代 | 文档说明限制；无私有 API |
| Android Credential Provider Passkey | `passkey/MonicaCredentialProviderService` | AuthenticationServices Passkey | 待实现 | 支持注册、认证、RP ID 校验、associated domains |
| TOTP 常驻通知 | `AutofillOtpNotificationService`, `NotificationValidatorService` | Widget/Live Activity/短时通知安全替代 | 开发中 | App 层已新增 Widget 安全快照边界：锁定状态不返回条目，解锁后只返回 TOTP 标题/issuer/account/剩余秒数和快捷入口摘要，不返回 TOTP code、secret、密码、note 正文、URL query、附件 hash 或本地路径；主 App 已通过 App Group 写入 `widget-snapshot-v1.json` 安全快照，`MonicaWidgetExtension` WidgetKit target 已接入 timeline provider 并只读取该脱敏快照，缺失/损坏时显示锁定态；锁屏小组件/Live Activity、签名真机 App Group/Widget 刷新验收仍待后续 |

## 同步、导入导出与外部格式

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| WebDAV 备份/恢复 | `webdav/`, `WebDavBackupScreen`, `SyncBackupScreen` | `MonicaSync` WebDAV | 已实现 | 上传、下载、SHA-256 校验、恢复前打开验证 |
| OneDrive | `OneDriveBackupScreen`, MSAL config | `CloudFileProvider` OneDrive adapter | 已实现，待真机验收 | `MonicaSync` 已新增通用 `CloudFileProvider` 契约和 OneDrive Graph REST app-folder provider；生产配置已写入 OneDrive MSAL client id `2aaf8c2c-b817-4085-9517-586a4a113dfc` 与 redirect URI `msauth.com.monica-pass.monica://auth`，主 App `Info.plist` 已注册 `msauth.com.monica-pass.monica` URL scheme，App 生产环境只注入 OneDrive provider；App 层已接入官方 MSAL 2.12.1 `MSAL.xcframework`，生产 `DefaultAppOneDriveMSALAuthenticationService` 支持真实交互登录、账号选择、redirect URL 回调、silent token refresh、登出清理本 app MSAL cache，并作为 `OneDriveCloudFileProvider` 的 `OneDriveAccessTokenProvider` 注入；设置页已提供 OneDrive 登录/退出、刷新文件、上传当前 vault 和下载预览入口；OneDrive provider 可通过 Microsoft Graph app folder 执行文件列表、下载、上传和带 If-Match revision 的条件覆盖写回，状态文案只显示 provider、清洗文件名和字节数，不泄漏 OAuth token、remote id/path、etag/revision、hash 或文件内容；真实 Microsoft 账号签名真机登录、真实 Graph 网络验收、云端恢复确认 UX 和真实 Graph 冲突 UX 仍待后续验收 |
| Google Drive | KeePass Google Drive browser | `CloudFileProvider` Google Drive adapter | 后置 | 按当前产品口径，Google Drive 暂不作为 feature 实现；`GoogleDriveCloudFileProvider` 仅保留编译期边界并对列表/下载/上传/覆盖写回返回 unsupported，生产环境不注入 Google Drive provider；Google Sign-In/Drive API 浏览/创建、云端恢复确认、冲突处理和签名真机验收均后置 |
| Android 备份包 | `DataExportImportViewModel.exportZipBackup/importZipBackup`, `WebDavHelper.createBackupZip/restoreFromBackupFile` | `AndroidBackupCodec` + App 迁移入口 | 开发中 | Storage 层已支持 Android ZIP 新版 `folders/<分类>/passwords/authenticators/bank_cards/documents/notes/passkeys/*.json` 核心条目导入，并可导出 Android 风格 ZIP；已兼容旧版 ZIP 内 `*_password.csv`、`*_totp.csv`、`*_cards_docs.csv`、`*_notes.csv` 兜底导入；已解析 `attachments/attachments_meta.json` 附件 manifest 元数据、blob 路径和 `attachments/*.enc` 密文 bytes，并在 App 预览文案中显示附件数量；确认导入时已把附件元数据写入 MDBX metadata repository，`parentPasswordId` 会 remap 到新建 iOS login entry id，附件密文保存到通用本地附件内容仓库且 metadata 记录 source/downloadState/wrapped CEK/localPath；导入后的附件引用已进入 Vault 页一等列表，支持搜索、软删除和恢复，并可检查/读取本地密文 blob；已补 Android 本地附件 AES-256-GCM raw CEK 解密、临时预览文件物化与 QuickLook 预览入口；App 层已可对已下载附件重新加密并写回本地 blob，同时更新 metadata 大小/hash/storageMode/downloadState；Storage/App 已支持在调用方提供 Android MDK/legacy wrapping key 时解包 `MDK|`/legacy wrapped CEK 并预览本地 `.enc` 附件；已识别 Android `MONICA_ENC_V1`/`.enc.zip` 加密备份，无密码时进入设置页密码输入提示，有密码时按 Android AES-256-GCM + PBKDF2-HMAC-SHA256(100000) 格式解密后复用 ZIP 导入路径，密码错误或文件损坏会返回明确失败并允许重试；设置页已接入 `.zip` 文件选择、加密备份密码提示、预览后确认导入当前 vault、导出 Android 风格 ZIP；Android MDK 自动迁移/导入、附件迁移/同步、回收站/配置恢复待接入 |
| CSV 导入导出 | import/export screens | CSV importer/exporter | 已实现 | Storage 层 CSV 编解码、字段映射、错误报告脱敏已实现；设置页已接入 iOS 文件导入/导出、预览后确认导入当前项目 |
| KDBX/KeePass | `LocalKeePass*`, `keepass/` | KDBX 读写兼容 | 已实现 | iOS 已完成 KDBX3/KDBX4 读写主链路：格式识别与解锁预检、password/key file 候选重试、KDBX3 AES-KDF、KDBX4 AES-KDF/Argon2d/Argon2id、AES-256/ChaCha20/Twofish payload、KDBX3 hashed block stream、KDBX4 header hash/HMAC 与 HMAC block stream、KDBX3 Salsa20 与 KDBX4 ChaCha20 inner protected value stream、KDBX4 inner header binary pool、KeePass XML snapshot 导入、密码/TOTP/notes/custom fields/附件 decoded 读取、KDBX3/KDBX4 snapshot writeback、本地源原文件替换、OneDrive/CloudFile 云源条件覆盖写回、原始 header bytes 复用、附件替换/新增/删除写回、回收站条目按 PreviousParentGroup 原分组恢复写回；Storage 和 App 回归测试覆盖真实 Argon2id、Twofish fixture 与写回读回闭环；状态文案不泄漏数据库密码、key file、派生 key、payload、XML、decoded password、TOTP secret、notes、附件内容、remote id 或 revision；KDB 旧格式仍按产品口径提示先转换为 .kdbx，OneDrive 真实 MSAL App 闭环已接入，签名真机/真实 Graph 网络验收属于云能力验收后续，不再阻塞 KDBX 本身 |
| Bitwarden 同步 | `bitwarden/`, `SyncQueue` | Bitwarden 双向同步 | 开发中 | `MonicaSync` 已新增 `BitwardenSyncProvider`、远端 vault/item/send snapshot、push mutation、冲突摘要和默认未登录 provider；App 层已接入 provider 注入，可执行 pull preview 与本地 Send push，开发者诊断只显示计数级状态；真实 Bitwarden OAuth/API、vault/folder/password/TOTP 双向映射、远端 ID 持久化、删除同步、附件同步、冲突合并 UI 和签名真机验收仍待后续 |

KDBX/KeePass 当前收尾备注：KDBX 不再作为 Android 对齐阻塞项；iOS 当前已覆盖现代 KDBX4 默认 Argon2id、Argon2d、AES-KDF、KDBX3 legacy AES-KDF、AES-256/ChaCha20/Twofish payload、KDBX3/KDBX4 block stream、inner protected value stream、inner header binary pool、key file 候选重试、本地与 OneDrive/CloudFile 云源 snapshot writeback、附件增删改写回、原始 header 复用、云 revision 冲突保护和 KeePass 原生回收站 PreviousParentGroup 恢复写回。后续 KDBX 只保留兼容性扩展与真机/真实云验收类工作，不再列为核心功能未实现。

## 管理、设置与商业化

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| 分类/快速筛选 | `CategoryDao`, `PasswordQuickFilter*` | iOS 分类、筛选、批量管理 | 开发中 | Vault 页已接入 iOS 横向快速筛选条，支持“全部 / 当前分类 / 收藏 / 回收站”入口，显示当前分类活动条目、收藏条目和已删除条目计数；点击收藏会清空搜索并联动所有一等条目的收藏筛选，点击回收站会隐藏活动列表并保留现有删除/恢复区块，锁库时筛选状态复位；Vault 页已接入分类管理条，支持分类列表、创建、切换、重命名和删除空分类，创建后自动切到新分类，切换会刷新全部条目列表并隔离搜索、收藏筛选和批量选择状态，含活动或回收站条目的分类会拒绝删除；Vault 页已接入批量管理条，支持当前可见结果的全选、选择态、批量软删除、回收站中批量恢复，以及跨分类批量移动；批量移动覆盖 `login/note/totp/card/identity/passkey/sshKey/apiToken/wifi/send/attachmentRef`，会从源分类移除、写入目标分类、保留原条目 ID，移动后刷新当前分类、清空搜索和选择态，并写入脱敏 `.moved` 时间线；当前分类列表/重命名/删除由 iOS Storage 会话索引维护，Rust UniFFI/MDBX 原生 project list/rename/delete 持久化仍待后续 |
| 堆叠分组 | `PasswordGrouping`, `StackedPasswordGroup` | iOS 分组/堆叠视图 | 开发中 | 密码页已接入“堆叠分组”开关，支持按网站 host 聚合并把 `www.` 归一到主域名；无 URL 条目回退标题分组；分组摘要显示条目数、标题预览和账号摘要，不包含密码；搜索、收藏和回收站筛选会联动分组结果，锁库时分组模式复位；备注/应用/自定义策略仍待后续 |
| 字段/页面定制 | `PageAdjustmentCustomizationScreen`, `PasswordFieldCustomizationScreen` | iOS 风格显示偏好 | 开发中 | 设置页已接入 iOS 风格显示偏好，支持密码列表账号字段显示/隐藏、网址字段显示/隐藏、舒适/紧凑卡片密度和底部导航文字显示/隐藏；偏好通过 UserDefaults 持久化，锁库不重置；完整字段顺序、页面重排和更多条目类型显示策略仍待后续 |
| 主题/图标 | `ColorSchemeSelectionScreen`, `IconSettingsScreen` | iOS 风格主题和图标设置 | 开发中 | 设置页已接入 iOS 外观偏好，支持跟随系统/浅色/深色颜色模式、Monica/绿色/蓝色/橙色强调色和密码列表图标彩色/单色/隐藏策略；偏好通过 UserDefaults 持久化，锁库不重置；App 根视图会套用 `preferredColorScheme` 和系统 tint，密码列表及堆叠分组会按图标策略显示；App Icon 变体和真机资产切换仍待后续 |
| 安全分析 | `SecurityAnalysisScreen` | 安全中心 | 开发中 | 设置页已显示安全中心第一版，统计弱密码、复用密码、泄露风险和重复登录条目数且不泄漏具体密码；泄露风险第一版使用本地 SHA-256 指纹库命中统计；已显示弱密码、复用、泄露风险和重复项的修复建议列表；在线泄露库同步、历史版本和自动修复动作待后续 |
| 重复项清理 | `DedupEngineScreen` | 安全中心重复项合并 | 开发中 | 已在安全中心显示重复登录条目摘要和合并预览，按标题/用户名/URL 去空白小写后分组；支持合并预览后软删除重复项并可从回收站恢复；支持忽略重复组并从摘要/预览中隐藏，设置页可恢复已忽略重复项；支持最近一次重复项合并的专门撤销入口；会话内登录 CRUD 操作时间线已接入安全中心；完整跨会话操作历史和版本恢复待后续 |
| 密码历史/时间线 | `TimelineScreen`, `PasswordHistoryDao`, `OperationLogDao` | 历史版本和操作时间线 | 开发中 | App 会话内已记录 `login/note/totp/card/identity/passkey/sshKey/apiToken/wifi/send` 条目创建、更新、删除、恢复操作时间线，并记录 `attachmentRef` 删除、恢复、QuickLook 预览准备和内容替换更新操作时间线；事件只包含动作、类型、条目 ID、标题和时间，不包含密码、用户名、笔记正文、TOTP seed、卡号/CVV、证件号、Passkey/SSH 私钥引用、API token、Wi-Fi 密码、Send 内容、附件 hash、wrapped key、本地密文路径、密文内容或附件明文等秘密；持久化历史版本、跨会话审计和版本恢复待后续 |
| Plus/支付 | `plus/`, `MonicaPlusScreen`, `PaymentScreen` | Android 同口径资源按钮本地解锁 Plus；不进入 StoreKit/IAP | 开发中 | 已按 Android `PaymentScreen(onActivatePlus)` 口径改为点击“激活 Plus”按钮后由资源 unlock service 本地解锁，无需购买或恢复购买；Settings 显示 Plus 激活/关闭状态和 Android 同口径功能权益列表，状态文案不泄漏 transaction、receipt、license、资源标识或其它凭据；旧 StoreKit 2 购买/恢复路径已从活动 App 层移除；App 层已新增 Plus 资源解锁权益持久化，生产环境通过 `UserDefaultsAppPlusEntitlementStore` 读取/写入资源解锁态，重新创建 session 后仍可恢复 Plus 激活状态，关闭 Plus 会同步写回未激活；真实资源包校验/映射和签名真机验收仍待后续 |
| 权限管理 | `PermissionManagementScreen` | iOS 权限状态中心 | 开发中 | 设置页已显示相机、AutoFill、通知、App Group、Keychain 状态中心；相机读取系统授权状态，通知读取 `UNUserNotificationCenter` 授权状态，AutoFill/App Group/Keychain 读取当前 App 配置状态；相机/通知提供 iOS App 设置入口；签名真机 entitlement 校验待后续 |
| 开发者设置 | `DeveloperSettingsScreen` | iOS debug/diagnostics | 开发中 | 设置页已显示脱敏诊断中心，覆盖主存储、MDBX 桥接、App Group、本机标识脱敏值、AutoFill 索引状态和 WebDAV 同步状态摘要；详细同步日志、fixture 导入和导出诊断包待后续 |

## 扩展与设备范围

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| 快捷入口 | Quick Tile, launcher aliases | Shortcuts/App Intents | 开发中 | App 会话已新增快捷入口安全摘要搜索、打开编辑器和 App Group 快照边界：解锁后可跨 login/note/totp/card/identity/passkey/sshKey/apiToken/wifi/send 生成 Shortcuts 可展示的 title/subtitle/searchableText，并按条目类型切换到对应 tab 打开编辑器；主 App 会在 App Group 可用时写入 `shortcuts-snapshot-v1.json`，只包含脱敏条目摘要和 `monica://shortcut/<kind>/<id>` deep link；主 App target 已新增 iOS 18+ `AppIntents` 实体查询和 `OpenMonicaShortcutEntryIntent`，只读 App Group 快照并通过系统 `OpenURLIntent` 打开条目 URL；App 根视图已接入 `monica://shortcut/...` URL handler，回到 App 后会路由到对应条目编辑器；摘要和快照不会包含 login password、note body、TOTP secret/code、API token、Wi-Fi password、Send body、私钥引用、附件 hash/wrapped key/localPath、URL query 或附件内容；快捷指令 UI 真机验证、复制动作、iOS 17 降级动作和签名真机 deep link 验收仍待后续 |
| 分享/导入 | Android intents/file picker | Share/Action Extension | 开发中 | 已新增 `MonicaShareExtension` target、Info.plist、App Group entitlements 和 ItemProvider 解析边界；Share extension 可接收 URL、纯文本和文件 URL，写入 App Group `share-inbox-v1`，manifest 只保存类型、mediaType、清洗文件名和相对内容路径，不包含 URL query、共享文本正文、源文件绝对路径或文件内容；App 会话可从 inbox 读取 pending items 并复用现有导入流创建 login/note/attachmentRef，导入成功后清空 inbox；签名真机 Share Sheet 唤起、更多 UTType/data item 兼容、二维码导入和冲突 UI 仍待后续 |
| 小组件 | Android 通知/快捷状态 | iOS Widget | 开发中 | 已新增 `MonicaWidgetExtension` WidgetKit target、App Group entitlements、timeline provider 和安全快照 reader；主 App production 会在 App Group 可用时注入 `AppWidgetSnapshotFileStore`，创建/打开/锁库和全量条目刷新会写入锁定态/解锁态摘要；Widget 只显示保险库状态、总数、TOTP 标题/issuer/account/剩余秒数或快捷入口摘要，不读取 vault、不显示 TOTP code/secret、密码、notes、URL query、附件 hash/localPath；签名真机安装、系统刷新策略、锁屏小组件和 Live Activity 仍待后续 |
| iPad | Android 大屏适配 | iPhone 优先 + iPad 自适应 | 开发中 | iPad 不崩溃、不遮挡；后续再做一等分栏体验 |
| Apple Watch | 无直接 Android 等价 | 后置 | 待实现 | 本轮不作为完成条件 |
