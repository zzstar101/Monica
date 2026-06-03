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
| TOTP 常驻通知 | `AutofillOtpNotificationService`, `NotificationValidatorService` | Widget/Live Activity/短时通知安全替代 | iOS 原生替代 | 不常驻暴露敏感验证码；用户主动开启后可快速查看 |

## 同步、导入导出与外部格式

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| WebDAV 备份/恢复 | `webdav/`, `WebDavBackupScreen`, `SyncBackupScreen` | `MonicaSync` WebDAV | 已实现 | 上传、下载、SHA-256 校验、恢复前打开验证 |
| OneDrive | `OneDriveBackupScreen`, MSAL config | `CloudFileProvider` OneDrive adapter | 待实现 | 登录、浏览、创建/打开、备份、恢复 |
| Google Drive | KeePass Google Drive browser | `CloudFileProvider` Google Drive adapter | 待实现 | 登录、浏览、创建/打开、备份、恢复 |
| Android 备份包 | `DataExportImportViewModel.exportZipBackup/importZipBackup`, `WebDavHelper.createBackupZip/restoreFromBackupFile` | `AndroidBackupCodec` + App 迁移入口 | 开发中 | Storage 层已支持 Android ZIP 新版 `folders/<分类>/passwords/authenticators/bank_cards/documents/notes/passkeys/*.json` 核心条目导入，并可导出 Android 风格 ZIP；已兼容旧版 ZIP 内 `*_password.csv`、`*_totp.csv`、`*_cards_docs.csv`、`*_notes.csv` 兜底导入；已解析 `attachments/attachments_meta.json` 附件 manifest 元数据、blob 路径和 `attachments/*.enc` 密文 bytes，并在 App 预览文案中显示附件数量；确认导入时已把附件元数据写入 MDBX metadata repository，`parentPasswordId` 会 remap 到新建 iOS login entry id，附件密文保存到通用本地附件内容仓库且 metadata 记录 source/downloadState/wrapped CEK/localPath；导入后的附件引用已进入 Vault 页一等列表，支持搜索、软删除和恢复，并可检查/读取本地密文 blob；已补 Android 本地附件 AES-256-GCM raw CEK 解密、临时预览文件物化与 QuickLook 预览入口；App 层已可对已下载附件重新加密并写回本地 blob，同时更新 metadata 大小/hash/storageMode/downloadState；Storage/App 已支持在调用方提供 Android MDK/legacy wrapping key 时解包 `MDK|`/legacy wrapped CEK 并预览本地 `.enc` 附件；已识别 Android `MONICA_ENC_V1`/`.enc.zip` 加密备份，无密码时进入设置页密码输入提示，有密码时按 Android AES-256-GCM + PBKDF2-HMAC-SHA256(100000) 格式解密后复用 ZIP 导入路径，密码错误或文件损坏会返回明确失败并允许重试；设置页已接入 `.zip` 文件选择、加密备份密码提示、预览后确认导入当前 vault、导出 Android 风格 ZIP；Android MDK 自动迁移/导入、附件迁移/同步、回收站/配置恢复待接入 |
| CSV 导入导出 | import/export screens | CSV importer/exporter | 已实现 | Storage 层 CSV 编解码、字段映射、错误报告脱敏已实现；设置页已接入 iOS 文件导入/导出、预览后确认导入当前项目 |
| KDBX/KeePass | `LocalKeePass*`, `keepass/` | KDBX 读写兼容 | 开发中 | Storage 层已按 Android `KeePassFormatInspector` 口径识别 KDBX、旧版 KDB 和未知格式；旧 KDB 会提示需先在 KeePassDX/KeePassXC 另存为 `.kdbx`；KDBX 预览会解析公开文件头中的 KDBX 3/4 版本摘要，进入等待密码/密钥文件解锁状态；Storage 已解析 KDBX4 公开 header 中 cipher、compression、KDF 参数摘要，设置页会显示如 `AES-256，GZip，Argon2id` 的脱敏算法诊断，不读取或显示数据库密码、key file 内容、派生 key 或明文字段；Storage 已新增严格 KDBX payload envelope 解析，可在遇到完整 end-of-header 后切分 header byte range、公开 TLV header fields 和 encrypted payload bytes，并用脱敏摘要显示 header/payload 字节数，后续真实 crypto 层可直接消费该边界；Storage 已结构化解析 KDBX4 VariantDictionary KDF 参数，覆盖 Argon2d/Argon2id 的 salt、memory、iterations、parallelism、version 以及 AES-KDF 的 seed/rounds，display 摘要只显示非秘密数值，不显示 salt、seed、数据库密码、key file 内容或派生 key；设置页已接入 KDBX 文件检查、数据库密码输入、密钥文件选择和解锁输入预检，预检只保留脱敏凭据摘要且不会写入当前 MDBX vault；Storage 已按 Android `KeePassCredentialSupport` 口径为 key file 构建 raw、XML `<Data>`、64 位 hex text 和 `sha256(raw)` key material 变体，并生成 `password-only`、`key-only`、`empty-password+key`、`password+key` 候选 label；密码 + key file 场景也会先尝试 `password-only` 再尝试 key file 组合，预检文案只显示候选数量、不泄漏数据库密码或 key file 内容；Storage 已新增候选尝试 reader，会按上述 label 顺序把 resolved password/key material 交给真实 reader，遇到 `.invalidCredential` 继续尝试，全部失败时只返回脱敏 label 摘要，App 只读预览已接入该管线；Storage 已新增可注入 `KeePassDatabaseReader`、脱敏 `KeePassUnlockCredentials` 和只读分组/条目 snapshot 模型，App 层可用 fake reader 预览只读树结构并保持 MDBX vault 不变；Storage 已新增解密后 KeePass XML reader，可把 XML 中的 Group/Entry/String/Binary 映射为只读 snapshot，保留分组路径、回收站标记、decoded password、TOTP、Notes、StringFields 和 decoded attachment bytes，默认 reader 会先尝试原始 XML，也支持对解密后的 GZip XML payload 解压后继续解析，普通加密 KDBX 仍返回“解码器尚未接入”；Storage/App 已新增只读导入计划预览，可把 snapshot 中活动条目和回收站条目都归入可预览候选，并用 `isDeleted` 标记回收站候选，设置页显示脱敏候选/跳过摘要；导入计划会统计仍待解码的密码字段、TOTP 和附件数量，已由 reader 提供的 decoded password/TOTP secret/attachment content 不再计入 pending，设置页与确认导入结果会显示这些 pending 能力且不泄漏数据库密码、key file 内容或 decoded secret/attachment content；App 已支持确认导入只读计划中的登录条目元数据，写入 title/username/url，并在 `KeePassDatabaseReader` snapshot 已提供 `decodedPassword` 时写入 iOS login password，未提供时继续留空并显示待解码能力；reader 提供 KeePass Notes 与 StringFields 时，Storage 会保留脱敏只读字段契约，App 会合并写入 iOS login notes，成功状态只显示计数、不泄漏字段值；导入时会把 KeePass 分组路径映射到 iOS 分类/项目（如 `/Work/Clients` -> `KeePass / Work / Clients`），并保留本次 KeePass entry/group 原生 ID 到 iOS login/category 的内部脱敏映射；带 `hasTotp` 的 KeePass 候选会在同一分类创建 iOS TOTP 条目，reader 提供 decoded TOTP secret 时导入 secret/issuer/account/period/digits/algorithm，未提供时创建空 secret 占位，并把 KeePass entry UUID 映射到 iOS login 与 TOTP 条目 ID；带附件摘要的 KeePass 候选会创建 iOS `attachmentRef` 元数据，关联到新建 login；reader 提供 decoded attachment bytes 时会保存到本地附件内容仓库，metadata 使用 `storageMode=keepass-kdbx-decoded-content`、`downloadState=downloaded` 并支持 QuickLook 直接预览；未提供 bytes 时仍创建 `storageMode=keepass-kdbx-placeholder`、`downloadState=pending-kdbx-decode` 占位；两种路径都会把 iOS 附件 ID 写入本次导入引用映射；回收站候选会创建对应 login/TOTP/附件元数据后立即通过 iOS repository 软删除，`AppKeePassImportedEntryReference.importedAsDeleted` 记录源 KeePass 条目到 iOS 已删除元数据的映射；导入后切到首个导入分类并清理 KDBX 文件、密码和 key file 临时状态；Storage 默认 reader 已通过真实加密 KDBX3 AES fixture 端到端完成 AES-KDF、master key、AES-CBC payload、KDBX3 hashed block stream 和 XML snapshot 读取；Storage 已新增 KDBX4 header hash/HMAC 校验与 HMAC block stream unwrap，可在 payload cipher 前校验并拼接 KDBX4 encrypted payload blocks；Storage 已修正 KDBX4 官方 AES-KDF UUID 并通过真实 KDBX4 AES-KDF fixture 端到端完成 header authentication、HMAC block unwrap、AES-CBC payload 和 XML snapshot 读取；Storage 已通过真实加密 KDBX4 AES-KDF + XML key file fixture 验证候选重试管线，可从 `password-only`、raw key file 失败继续到 XML `<Data>` key material 成功读取 snapshot；Storage 已支持 KDBX4 ChaCha20 payload cipher，按 32-byte master key 与 12-byte encryption IV 解密 payload 后继续复用 HMAC block、XML/GZip、inner header 与导入链路；Storage 已接入 KDBX3 Salsa20 与 KDBX4 ChaCha20 inner protected value stream，且默认 reader 已支持从 KDBX4 解密后 inner header 读取 inner random stream ID/key 与 field 3 binary attachment pool，再把后续 XML 或 GZip XML 交给只读 snapshot reader；XML 中 `Protected="True"` 的 password/custom/TOTP 字段可按 inner stream 顺序解保护后进入只读 snapshot，XML 中引用 inner header binary pool 的附件可作为 decoded attachment bytes 进入既有导入/QuickLook 路径；Storage 已接入官方 PHC Argon2 reference C package，支持 KDBX Argon2d/Argon2id KDF raw 32-byte derived key，并用 Argon2d/Argon2id 向量验证不泄漏 salt、composite key、derived key 或 encrypted payload；Storage 已通过真实 KDBX4 Argon2id fixture 验证 Argon2id KDF -> master key -> header HMAC -> HMAC block unwrap -> AES-CBC payload -> XML snapshot 读取链路；Storage 已接入官方 Twofish reference C target，支持 KDBX Twofish-CBC payload cipher，并通过真实 KDBX4 Twofish fixture 验证 header HMAC、HMAC block unwrap、Twofish-CBC payload 和 XML snapshot 读取链路；导入到 iOS 本地内容仓库后的 KeePass decoded 附件已可复用附件写回第一版进行内容替换和 metadata/hash 更新；Storage 已新增 KDBX block stream writer，可把 XML writeback payload 包装为 KDBX3 hashed block stream 或 KDBX4 HMAC block stream 并由现有 decoder 回读；Storage 已新增 GZip payload compressor 和 AES-256-CBC payload cipher writer，可把 XML/GZip/block stream payload 继续推进到 AES encrypted payload 层并由现有 decryptor 回读；Storage 已新增 KDBX4 payload section writer，可写出 header hash、header HMAC 和 HMAC block stream payload section 并由现有 decryptor 回读；KDBX header 生成、完整文件 assembly/原位 writeback、云文件源和 KeePass 原生回收站还原语义仍待后续 |
| Bitwarden 同步 | `bitwarden/`, `SyncQueue` | Bitwarden 双向同步 | 待实现 | 登录、vault/folder、密码、TOTP、Send、附件、冲突处理 |

KDBX/KeePass 进展备注：Storage 已新增 KDBX 解密输入上下文，将 payload envelope、结构化 KDF 参数、候选凭据 label 和 Android 同口径复合凭据 key material 组合为可注入 `KeePassKdbxPayloadDecryptor` 的输入。复合 key material 使用 SHA-256(password UTF-8) 与 resolved key file material 拼接后再 SHA-256；摘要只显示 KDBX 版本、payload 字节数、KDF、候选 label 和凭据组件类型，不泄漏数据库密码、key file 内容、salt/seed、复合 key 或 encrypted payload。默认 reader 已在普通加密 KDBX 分支先构造该上下文并调用占位 decryptor；真实 Argon2/AES-KDF 执行、payload/block 解密、附件写回/编辑和 KDBX 保存仍待后续。

KDBX/KeePass 进展备注：Storage 已新增 KDBX key deriver 边界并完成 AES-KDF transform 第一版。AES-KDF 使用 KDBX header 中的 32 字节 seed 对 32 字节 composite key 执行 AES-256 ECB rounds 变换，再 SHA-256 得到 derived key；测试使用 NIST AES-256 已知向量验证一轮变换。占位 payload decryptor 现在会先调用可注入 key deriver，再返回脱敏“payload 解密尚未接入”；Argon2d/Argon2id KDF、master key/payload/block 解密、真实加密 KDBX 导入和保存仍待后续。

KDBX/KeePass 进展备注：Storage 已新增 KDBX payload crypto header 输入边界，结构化保留 master seed、encryption IV、stream start bytes、inner random stream key/id 与 inner stream 算法摘要；摘要只显示长度和算法，不显示 master seed、IV、inner key、stream start bytes、KDF seed、数据库密码或 encrypted payload。Storage 同时新增 master key composer，使用 `SHA256(masterSeed + derivedKey)` 生成 KDBX master key material，并让占位 payload decryptor 在停止于 payload 解密前先完成 KDF 与 master key 组合；真实 Argon2d/Argon2id 执行、payload/block 解密、inner stream 解密、真实加密 KDBX 导入和保存仍待后续。

KDBX/KeePass 进展备注：Storage 已新增 KDBX payload cipher 边界并完成 AES-256-CBC 解密第一版。`DefaultKeePassKdbxPayloadCipher` 会用 KDBX master key 与 encryption IV 解密 AES payload，支持合法 PKCS#7 padding 剥离；ChaCha20 payload 已在后续节点补齐；Twofish payload 已接入官方 reference C target 并按 KDBX 32-byte master key、16-byte IV 做 CBC 解密；未知 cipher 仍返回明确脱敏 unsupported。默认 payload decryptor 会按 KDF -> master key -> payload cipher 顺序推进；这仍不声明 KDBX 保存已完成。

KDBX/KeePass 进展备注：Storage 已新增 KDBX3 hashed block stream decoder，支持解密后 payload 的 stream-start 校验、`UInt32 index + SHA-256 hash + UInt32 length + block data` block 校验、zero-hash/zero-length terminator 识别和 XML/GZip payload 拼接输出；默认 payload decryptor 现在可在 KDBX3 路径完成 KDF -> master key -> AES payload cipher -> hashed block stream。错误文案仍脱敏，不显示 stream start、block hash、encrypted payload、数据库密码、key file、derived/master key 或 XML 内容；KDBX4 header hash/HMAC 校验与 HMAC block stream unwrap 已接入；inner protected value stream、Argon2d/Argon2id 执行、真实加密 KDBX4 fixture 端到端导入和保存仍待后续。

KDBX/KeePass 进展备注：Storage 已新增 KDBX3 旧式 AES-KDF header 解析，按 KDBX3 field 5/6 读取 TransformSeed 与 TransformRounds，并把它们归入既有 `KeePassKdbxKdfParameters(.aesKdf)`，公开摘要显示 `AES-KDF` 与 rounds 但不泄漏 transform seed、encrypted payload、数据库密码、key file 或 derived key。KDBX4 VariantDictionary KDF 路径保持不变；KDBX4 HMAC block stream 已接入；inner protected value stream 和 KDBX 保存仍待后续。

KDBX/KeePass 进展备注：Storage 已用真实加密 KDBX3 AES fixture 验证默认 reader 端到端只读路径：KDBX3 header -> password-only composite key -> AES-KDF -> master key -> AES-256-CBC encrypted payload -> KDBX3 hashed block stream -> KeePass XML snapshot。解密后的 snapshot 会保留 KDBX3 header summary，并读取 decoded password 供既有只读导入/确认导入链路消费；display summary 继续不泄漏数据库密码、decoded password、transform seed、encrypted payload、derived/master key 或 XML 明文。KDBX4 HMAC block stream 已接入；inner protected value stream、Argon2d/Argon2id 执行、KDBX 保存和附件写回/编辑仍待后续。

KDBX/KeePass 进展备注：Storage 已新增 KeePass XML writeback payload writer 第一版，可把已解密/已编辑的 `KeePassReadOnlySnapshot` 重新序列化为 KeePass XML payload，保留分组层级、条目 UUID、回收站分组标记、密码/TOTP/Notes/StringFields 和 decoded attachment binaries；writer 摘要只显示分组/条目/附件数量，不显示数据库密码、decoded password、TOTP secret、自定义字段值或附件明文。该节点只补齐 KDBX 保存前的明文 XML payload 层，不声明 KDBX header、压缩、block stream、HMAC、payload cipher 或原文件原位保存/writeback 已完成。

KDBX/KeePass 进展备注：Storage 已新增 KDBX block stream writer 第一版，可把 XML writeback payload 包装为 KDBX3 hashed block stream（stream start + block SHA-256 + zero terminator）或 KDBX4 HMAC block stream（per-block HMAC + terminator），并用现有 decoder 回读验证；错误文案不泄漏 XML 明文、stream start bytes 或 HMAC base key。该节点只补齐 KDBX 保存链路的 block stream 层，不声明 KDBX header 生成、GZip 压缩写回、payload cipher 加密、完整文件 assembly、原文件原位保存或云文件源 writeback 已完成。

KDBX/KeePass 进展备注：Storage 已新增 KDBX writeback 压缩/加密子层第一版：`KeePassGzipPayloadCompressor` 可生成标准 GZip payload 并由现有 reader 解压回 KeePass XML；`DefaultKeePassKdbxPayloadCipher.encryptPayload` 已支持 AES-256-CBC + PKCS#7 padding 写回，并可由现有 AES decryptor 还原。错误文案不泄漏 XML payload、master key、IV 或明文；ChaCha20/Twofish payload 加密、KDBX header 生成、完整文件 assembly、原文件原位保存或云文件源 writeback 仍待后续。

KDBX/KeePass 进展备注：Storage 已新增 KDBX4 payload section writer 第一版，可按 KDBX4 口径写出 `SHA256(header bytes)`、header HMAC 和 HMAC block stream payload section，并由现有 `DefaultKeePassKdbxPayloadDecryptor` 完成 header hash/HMAC 校验、HMAC block unwrap 后交给 payload cipher；错误文案和测试输出不泄漏 header bytes、encrypted payload、master seed 或 derived key。该节点仍不声明 KDBX header 生成、完整文件 assembly、原文件原位保存或云文件源 writeback 已完成。

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
| Plus/支付 | `plus/`, `MonicaPlusScreen`, `PaymentScreen` | Android 同口径资源按钮本地解锁 Plus；不进入 StoreKit/IAP | 开发中 | 已按 Android `PaymentScreen(onActivatePlus)` 口径改为点击“激活 Plus”按钮后由资源 unlock service 本地解锁，无需购买或恢复购买；Settings 显示 Plus 激活/关闭状态和 Android 同口径功能权益列表，状态文案不泄漏 transaction、receipt、license、资源标识或其它凭据；旧 StoreKit 2 购买/恢复路径已从活动 App 层移除；Plus 权益持久化、真实资源包校验/映射和签名真机验收仍待后续 |
| 权限管理 | `PermissionManagementScreen` | iOS 权限状态中心 | 开发中 | 设置页已显示相机、AutoFill、通知、App Group、Keychain 状态中心；相机读取系统授权状态，通知读取 `UNUserNotificationCenter` 授权状态，AutoFill/App Group/Keychain 读取当前 App 配置状态；相机/通知提供 iOS App 设置入口；签名真机 entitlement 校验待后续 |
| 开发者设置 | `DeveloperSettingsScreen` | iOS debug/diagnostics | 开发中 | 设置页已显示脱敏诊断中心，覆盖主存储、MDBX 桥接、App Group、本机标识脱敏值、AutoFill 索引状态和 WebDAV 同步状态摘要；详细同步日志、fixture 导入和导出诊断包待后续 |

## 扩展与设备范围

| Android 功能域 | Android 来源 | iOS 目标实现 | 当前状态 | 验收标准 |
| --- | --- | --- | --- | --- |
| 快捷入口 | Quick Tile, launcher aliases | Shortcuts/App Intents | 开发中 | App 会话已新增快捷入口安全摘要搜索和打开编辑器第一版：解锁后可跨 login/note/totp/card/identity/passkey/sshKey/apiToken/wifi/send 生成 Shortcuts 可展示的 title/subtitle/searchableText，并按条目类型切换到对应 tab 打开编辑器；摘要不会包含 login password、note body、TOTP secret、API token、Wi-Fi password、Send body、私钥引用、附件 hash/wrapped key/localPath 等秘密；系统 AppIntents 注册、快捷指令 UI、复制动作和签名真机验证仍待后续 |
| 分享/导入 | Android intents/file picker | Share/Action Extension | 待实现 | 从文件、URL、二维码、文本导入到当前 vault |
| 小组件 | Android 通知/快捷状态 | iOS Widget | 待实现 | 安全显示 TOTP/快捷状态，不泄漏秘密 |
| iPad | Android 大屏适配 | iPhone 优先 + iPad 自适应 | 开发中 | iPad 不崩溃、不遮挡；后续再做一等分栏体验 |
| Apple Watch | 无直接 Android 等价 | 后置 | 待实现 | 本轮不作为完成条件 |
