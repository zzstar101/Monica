# 调研记录

## 仓库现状

- Android App 是当前主客户端，目录是 `Monica for Android`。
- Android 端使用 Kotlin、Jetpack Compose、Room，并在 `MonicaApplication` 中启动 Koin；但大量业务依赖仍在 `MainActivity` 中手动装配。
- Android 功能范围很宽，包括 WorkManager、WebDAV、KeePass、Bitwarden、Passkey、AutoFill、IME、Accessibility、附件和 TOTP。
- `PasswordDatabase.kt` 是当前 Room 本地数据库，版本号已经到 68，说明数据模型经历了长期演进。
- `mdbx` 是 Monica 新本地加密 vault 格式的 Rust workspace。
- `mdbx-doc` 是 MDBX 的规范文档目录，负责定义产品、存储、同步、安全和 schema 行为。
- `MonicaDocs` 和 `documentation/website` 是文档站/官网，不是主应用运行时代码。

## iOS 平台约束

- iOS AutoFill 需要通过 AuthenticationServices 的 Credential Provider Extension 实现，核心类包括 `ASCredentialProviderViewController`。
- AutoFill 的 credential identity 通过 AuthenticationServices 的身份存储暴露给系统，而不是直接从主 SwiftUI App 暴露。
- 主 App 和 Extension 之间需要 App Group 共享数据，但共享内容必须加密或压缩成最小必要元数据。
- 生物识别解锁应基于 LocalAuthentication，并结合 Keychain / Secure Enclave 做密钥包装，而不是保存明文 vault key。
- iOS 后台能力比 Android WorkManager 严格，WebDAV 备份应设计为前台优先，后台同步只能尽力而为。
- Passkey 与 AuthenticationServices、associated domains、relying-party identifier 绑定，Android 的 Passkey 行为不能直接复制。

## 迁移判断

- 直接做 Android 全功能对齐，范围过大，不适合作为第一版 iOS。
- 长期最稳的路线是 SwiftUI 原生客户端 + 跨平台数据格式测试 + 尽可能复用 Rust MDBX 核心。
- 第一版不应同时承担 legacy Room 兼容、完整 MDBX、Bitwarden 双向同步、附件、Passkey 和高级 AutoFill。
- 已决定首版采用 MDBX 优先路线：iOS 不先做临时 SQLite 主存储。
- 已决定 Rust/Swift 桥接采用 UniFFI。UniFFI 官方文档说明 Swift binding 可由 Rust library/UDL 生成，并支持 Swift 原生类型映射。
- 已决定最低支持 iOS 17+、iPhone 优先。
- 已决定 AutoFill 使用 App Group 加密索引：主 App 维护加密后的域名/标题/账号索引，Extension 解锁后读取。
- 已决定 Passkey、KeePass、Bitwarden、附件、Plus 后置；当前 Android 口径下 Plus 是资源按钮解锁，不进入 IAP。

## MDBX 规范约束

- MDBX 实现必须先读 `mdbx-doc`，并以其中编号文档为唯一规范来源。
- iOS FFI 的最小测试接口也必须保留 `project -> entry` 层级，不能为了测试方便做无 project 的平铺 entry。
- 创建 vault 时必须使用 v1 schema，保留 `projects`、`entries`、`attachments`、`attachment_chunks`、commit DAG、tombstone、snapshot、key epoch 等结构。
- 首阶段只暴露 Multi Type，但不能削弱 Argon2id、认证加密、Unicode 密码规范化等安全要求。
- UniFFI 生成物需要和可链接的 iOS device/simulator 二进制一起接入；在 XCFramework 未完成前，生成 Swift/header/modulemap 应留在 `Generated/MDBXUniFFI`，不直接进入 SwiftPM `Sources`。

## 当前技术验证状态

- Rust 侧新增 `mdbx-ios-ffi` facade，最小 API 覆盖 create/open/unlock/project/login entry create/list/update/delete/list-deleted/restore、note entry create/list/update/delete/list-deleted/restore、TOTP entry create/list/update/delete/list-deleted/restore，以及本地 `security_key` unlock setup/open。
- smoke test 使用 Unicode 主密码，并保留 `project -> entry` 层级。
- Swift 侧已创建 `MonicaMDBX` package，并接入 iOS-only UniFFI binding 与 `mdbx_ios_ffiFFI` binary target。
- `MonicaMDBX` wrapper 已将生成类型封装为 Swift 友好的 `MonicaMDBXVault`、`MonicaMDBXProject`、`MonicaMDBXLoginEntry`、`MonicaMDBXNoteEntry`、`MonicaMDBXTotpEntry` 和 `MonicaMDBXCardEntry`，主 App 不直接依赖生成的 UniFFI API。
- wrapper 提供 `MonicaMDBXTechnicalVerifier.runProjectScopedLoginRoundTrip`，用于验证 iOS 创建 vault、创建 project、写入 login entry、重新打开、读取 entry 的链路；安全笔记、TOTP、银行卡和证件元数据也已有独立 UniFFI round-trip XCTest 覆盖。
- 非 iOS SwiftPM 测试环境会显式返回 `unavailableOnCurrentPlatform`，避免 macOS `swift test` 误用 iOS-only FFI。
- `MonicaTests` Xcode test target 已接入 `MonicaMDBX`，可在 iOS simulator 中自动执行 MDBX round-trip。
- `Monica` 共享 scheme 已配置 test action；命令行测试必须使用具体模拟器 destination，不能使用 generic simulator。
- `MonicaStorage` 现在依赖 `MonicaMDBX`，并通过 `LocalVaultEngine` 协议隔离 repository 与真实 FFI，便于单元测试和后续替换实现。
- `LocalVaultRepository` 当前只负责本地 vault create/open 的最小边界、`.mdbx` 描述、基本输入校验和 unlocked session 返回；条目 repository、scene phase 锁定和基础 5 分钟 idle timeout 已接入 App。
- `LocalVaultEntryRepository` 已定义条目管理边界，覆盖 project-scoped login entry、note entry、TOTP entry、card entry 和 identity entry 的 create/list/update/delete/list-deleted/restore/favorite；当前 App UI 已接入密码登录、安全笔记、TOTP、银行卡和证件元数据的 create/list/search/select/edit/delete/restore/favorite，并在 App 会话层支持收藏优先排序和 Favorites Only 筛选，筛选可与搜索叠加。`MonicaCore` 已提供 RFC 6238 TOTP 基础生成器，并由 App 会话模型用于根据存储 seed 生成当前验证码；TOTP `otpauth://` URI 导入和相机 QR 扫描入口已接入 App 草稿表单，列表和选中态会按 entry period 显示每秒刷新的剩余时间。
- `MonicaStorage` 已定义 AutoFill 加密索引的最小 App Group 数据契约：`AutoFillEncryptedIndex` envelope 只包含 schema/version、vault id、key id、更新时间和加密记录；单条 `AutoFillEncryptedIndexRecord` 只包含 id、nonce、ciphertext、authentication tag，不包含明文域名、标题或账号。`AutoFillEncryptedIndexCodec` 使用 CryptoKit `AES.GCM` 加密/解密 `AutoFillCredentialIndexRecord`，payload 可包含标题、账号和 service identifiers，但只进入 ciphertext。`AutoFillIndexEncryptionKey` 当前强制 32 字节，后续由 Keychain/LocalAuthentication 解锁边界提供真实 key material。`FileAutoFillEncryptedIndexStore` 负责在 App Group container 下保存 `autofill-index-v1.json`，iOS 写入使用 complete file protection。`AutoFillCredentialIndexUnlocker` 会在解密前校验 vault id/key id；`AutoFillUnlockedCredentialIndex` 支持按 service identifier 域名/URL 匹配和按标题、账号、service identifier 搜索。
- App 层已开始消费 AutoFill 加密索引契约：`AppSessionModel` 能把当前登录条目的 title、username、url host/原始 URL 映射成 `AutoFillCredentialIndexRecord`，用注入的 `AutoFillIndexKeyMaterial` 和 `AutoFillEncryptedIndexStore` 生成密文索引；创建、更新、软删除和恢复登录条目后会自动刷新索引。测试已确认编码后的 envelope 不包含 `github.com`、账号或标题明文，并确认更新后的 service identifier 可从解密 payload 读回。
- `MDBXLocalVaultEngine` 现在会按 vault id 保留打开的 `MonicaMDBXVault` 引用，使同一个 unlocked session 能继续创建 project 和 entry；这只是进程内会话缓存，后续仍需要真实锁定窗口和资源释放策略。
- App 层 `AppSessionModel` 已通过 `LocalVaultRepository` 支持 create/open/lock 三个会话动作；主密码在 create/open 成功、create/open 失败或 lock 时会从内存中的表单字段清空，并通过 scene phase 做 inactive 隐私遮罩、background 锁定和 active 时 idle timeout 检查。自动锁定策略已抽象为固定预设，Settings 通过 Picker 写回会话模型，切换策略时会刷新活动窗口。
- `MonicaSecurity` 已建立 Keychain/LocalAuthentication 第一层边界：`WrappedVaultKey` 表示 Keychain 保护下的本地解锁 material，`WrappedVaultKeyStore` 抽象 Keychain 持久化，`MonicaLocalAuthenticator` 抽象本地认证，`VaultKeychainManager` 强制读取该 material 前先认证；`AutoFillIndexKeyMaterial` 表示 32 字节 AutoFill 索引加密 key material，`AutoFillIndexKeyStore` 抽象 Keychain 持久化，`AutoFillIndexKeychainManager` 强制读取索引 key material 前先认证并拒绝非 32 字节 key。App 侧已新增同步 AutoFill index key material provider/store 作为生产写索引路径，也新增了 `AppVaultKeychainService` 与 vault keychain 状态流：手动解锁后生成本地 security key material、调用 MDBX `setup_security_key` 注册到当前 vault，再保存到 Keychain；锁定后通过 LocalAuthentication 读取 material，并调用 `LocalVaultRepository.openVaultWithSecurityKey` 打开 remembered vault。这个路径不保存主密码，也不通过 resolver 还原主密码，符合 `mdbx-doc/03-security-spec.zh-CN.md` 中“生物识别只包裹更强底层秘密，不取代 vault secret”的要求。
- Vault 页已接入 `LocalVaultRepository`：
  - create 使用 App documents directory 写入 `<vault name>.mdbx`。
  - open 使用 SwiftUI file importer 选择 `.mdbx` 文件，并处理 security-scoped resource。
  - lock 目前清理当前 App 会话状态并被后台锁定/idle timeout 复用；真实 MDBX handle 生命周期释放策略仍需继续细化。
  - 解锁后可以创建密码条目并列出当前 project 下的 login entries；当前 UI 仍是验证雏形，不是最终条目管理体验。
  - 选择登录条目后可编辑标题、用户名、密码和 URL；保存会走 `LocalVaultEntryRepository.updateLoginEntry` -> `MonicaMDBXVault.updateLoginEntry` -> UniFFI -> Rust `EntryRepo::update`，保留同一 entry id 并产生 MDBX 侧 commit/object version。
  - 删除登录条目会走 Rust `EntryRepo::soft_delete`，写 tombstone 并从普通列表移到回收站；恢复会走新增的 `EntryRepo::restore`，将同一 entry id 重新标记为未删除并产生新的 commit/object version。恢复不会物理清理 tombstone，历史清理应留给 purge/compaction 策略。
  - 当前登录条目搜索是 App 会话内过滤，覆盖 title、username、url，不会触发新的 MDBX 查询；AutoFill 使用独立的加密索引文件，主 App 负责生成，后续 Extension 需要在认证后读取并在内存中搜索/匹配。
  - 安全笔记使用同一个 active project 和 `LocalVaultEntryRepository`，App 会话内支持 title/body 搜索、选择编辑、软删除到 `Deleted Notes` 和恢复；当前安全笔记不会进入 AutoFill 加密索引或 credential identities。
  - TOTP 已完成从 `mdbx-ios-ffi`、`MonicaMDBX`、`MonicaStorage` 到 App UI 的存储/编辑纵切；payload 字段按 `mdbx-doc/11-monica-pass-cli-development.zh-CN.md` 的 Android `TotpData` 兼容形状保存。TOTP seed 属于秘密型 payload，不应进入日志、明文索引、AutoFill 索引或用户不可见诊断输出。App UI 现在可通过 `MonicaCore.TotpGenerator` 根据存储 seed 显示当前验证码。
  - `MonicaCore.TotpGenerator` 使用 CryptoKit HMAC，当前覆盖 RFC 6238 TOTP 的 SHA1/SHA256/SHA512、Base32 secret 规范化、period 和 digits 校验；`TotpURIParser` 支持标准 `otpauth://totp` URI、issuer/account label fallback、period/digits/algorithm 解析和错误校验；App 层会把扫描得到的 QR payload 复用同一 parser 导入草稿，不落库；无效扫描会映射为用户可读提示，scanner sheet 会保留并恢复扫描；SwiftPM macOS 测试目标因此显式声明 `.macOS(.v13)`，与 Storage/Sync 的 CryptoKit 用法保持一致。
  - `MonicaCore.PasswordGenerator` 使用 Security framework 的 `SecRandomCopyBytes` 生成默认 20 位密码，默认包含大小写字母、数字和符号，不包含空白字符；生成逻辑会保证每个启用字符集至少出现一次。App 层只把生成结果填入新增登录或已选登录的密码草稿，不自动写入 MDBX，只有用户点击保存/新增后才落库。
  - 多类型条目收藏 MVP 已完成：`favorite` 存在 login/note/TOTP/card/identity entry 的加密/序列化 payload 中，不改 MDBX v1 `entries` schema，不新增明文列；create 默认未收藏，普通 update 保留已有收藏状态，专用 `set_*_favorite` 只改变收藏位并保留各类型 payload 字段。收藏状态不进入 AutoFill encrypted index、secret snapshot 或 credential identities；App 会话层默认把收藏项排在列表前面，并提供 Favorites Only 筛选，不改变 MDBX 存储格式。
  - 银行卡已完成从 `mdbx-ios-ffi`、`MonicaMDBX`、`MonicaStorage` 到 App UI 的存储/编辑纵切；payload 形状包含 `kind = card`、持卡人、卡号、有效期、CVV、issuer、network 和 notes。卡号、CVV、备注等字段作为 entry payload 处理，不进入明文列、AutoFill 加密索引、AutoFill secret snapshot 或 credential identities；App UI 列表摘要只显示 network/issuer/末四位。
  - 证件元数据已完成从 `mdbx-ios-ffi`、`MonicaMDBX`、`MonicaStorage` 到 App UI 的存储/编辑纵切；payload 形状包含 `kind = identity`、document type、full name、document number、issuer、country、issue date、expiry date 和 notes。证件号、备注等敏感字段作为 entry payload 处理，不拆明文列，不进入 AutoFill 加密索引、AutoFill secret snapshot 或 credential identities；App UI 列表摘要只显示证件类型、姓名、签发方和国家等非完整敏感信息。
- Xcode 26.5 和 iOS 26.5 device/simulator SDK 可用。
- 当前已使用 rustup 管理的 toolchain，并安装 `aarch64-apple-ios`、`aarch64-apple-ios-sim`、`x86_64-apple-ios`。
- 已新增并验证 XCFramework 构建脚本，输出 `Artifacts/MDBX/MonicaMDBXGenerated.xcframework`。
- 真机第一轮验证已确认：`xcodebuild` 能识别真实设备 `Evangelion`，`CODE_SIGNING_ALLOWED=NO` 的 `iphoneos` arm64 build 可成功完成，App、debug dylib 和 AutoFill Extension 产物均为 arm64 Mach-O，说明 device slice 和 `MonicaMDBXGenerated.xcframework` 的真机 slice 可编译/链接。实际安装/运行仍被签名阻塞：本机有 Apple Development identity `B6R6XP99R2`，但 Xcode Accounts 没有该 Team 的有效账号，且本地没有 `takagi.ru.monica` / `takagi.ru.monica.autofill` development provisioning profiles；未签名产物通过 `devicectl` 安装到设备时返回 `0xe800801c (No code signature found.)`。
- SwiftUI App 源码骨架已采用 iOS 17 Observation 路线，主界面先保留 vault / AutoFill / Settings 三个 iPhone-first tab。
- SwiftUI App 已接入 `MonicaMDBX`，Vault 页可以手动运行 MDBX 技术验证。
- AutoFill Extension 源码已使用 `ASCredentialProviderViewController`，锁定状态下对无 UI credential 请求返回 `ASExtensionError.Code.userInteractionRequired`；credential list 路径会从 App Group 读取加密索引和加密 secret snapshot，通过 `AutoFillIndexKeychainManager` 和 `DeviceOwnerLocalAuthenticator` 解锁 AutoFill index key，再按系统传入的 service identifiers 匹配并显示可搜索的元数据列表。用户选择记录后，Extension 会按 entry id 读取解密后的 username/password，构造 `ASPasswordCredential` 并调用 `extensionContext.completeRequest(withSelectedCredential:)` 交给系统填充。主 App 已有 `ASCredentialIdentityStore` 同步链路；下一步是真机签名环境验证 QuickType 展示、Credential Provider 和 App Group / Keychain access group。
- 主 App 已接入 `ASCredentialIdentityStore` 的第一条同步路径：App 会话通过平台无关的 `AppAutoFillCredentialIdentityStore` 生成和同步 identities，生产 adapter 使用 `ASCredentialIdentityStore.shared.replaceCredentialIdentities(_:completion:)` 写入 `ASPasswordCredentialIdentity`。当前 XCTest 已覆盖创建、更新、删除和恢复登录条目时 identity 列表保持同步；真实 QuickType 展示仍需要签名真机验证。
- `MonicaSync` 已开始 WebDAV 首版基础：WebDAV client 采用 transport 注入边界，测试中不触发真实网络，生产路径用 `URLSessionWebDAVTransport`。当前支持 Basic auth、PUT 上传、GET 下载、`X-Monica-Backup-SHA256` 完整性 header、`.mdbx.sha256` sidecar fallback、下载完整性校验和恢复预览。App 层通过 `AppWebDAVBackupService` adapter 接入，Settings 已能手动上传当前 active vault 文件、下载恢复预览并确认恢复。确认恢复会先把下载的备份写入临时候选 `.mdbx` 文件，并用恢复 vault 密码通过 `LocalVaultRepository.openVault` 打开验证；只有候选文件可打开时才释放 active vault session，并用 `FileManager.replaceItemAt` 原子替换本地 vault 文件。候选备份打不开时，本地 vault 文件和当前 unlocked session 都保持不变，恢复预览保留但恢复密码清空。这符合 `mdbx-doc` 中安全正确性、恢复测试、数据耐久性优先，以及 WebDAV/网盘提供方不可信且可能非原子的假设。App 层已把 WebDAV 认证失败、网络断开、完整性校验失败、远端文件缺失、服务端错误、超时、无法连接服务器和非 HTTP 响应映射为用户可读状态，避免直接暴露底层 HTTP/URLSession 技术文案。
- `Monica.xcodeproj` 已创建 App、AutoFill Extension 和 Tests target，并通过 iOS simulator Debug build/test。
- `AppSessionModel` 已接入 iOS scene phase 安全策略：`inactive` 时显示全屏隐私遮罩但保留 unlocked session，`background` 时立即执行锁定并清空活动 vault、条目列表、搜索、编辑状态和临时密码字段，`active` 时移除遮罩并检查 5 分钟 idle timeout。
- `MonicaCore` SwiftPM 当前覆盖 7 个用例，包含 iOS 17 / MDBX-first 基线、RFC 6238 TOTP 向量、Base32 secret 规范化、无效参数、`otpauth://` URI 导入、URI 错误校验、密码生成器策略约束和无效策略拒绝；`MonicaSecurity` SwiftPM 当前覆盖 7 个用例，包含 vault wrapped key 和 AutoFill index key 的认证读取边界；`MonicaStorage` SwiftPM 当前覆盖 22 个用例，包含 security key setup/open repository 委托、login/note/totp/card/identity entry repository、多类型 favorite repository、AutoFill 加密索引 save/load、缺失文件返回 nil、索引文件不包含明文 credential metadata、AES-GCM encrypt/decrypt round-trip、codec 输出不包含明文 credential metadata、解密索引域名匹配和搜索、AutoFill 加密 secret snapshot encrypt/decrypt、secret 文件不包含明文账号/密码；`MonicaSync` SwiftPM 当前覆盖 8 个用例，包含 WebDAV 上传、`.sha256` sidecar 上传、下载、Basic auth、header/sidecar 完整性校验、恢复预览和错误状态；`MonicaUI` SwiftPM 当前覆盖 1 个用例；Rust `mdbx-ios-ffi` 当前覆盖 10 个 smoke test，包含 login/note/totp/card/identity、security key 和多类型 favorite；`MonicaTests` 当前包括 MDBX login/note/totp/card/identity UniFFI round-trip、MDBX local security key setup/open round-trip、Storage entry repository 真实 MDBX round-trip、App vault session create/open/lock、App vault create/open 失败安全清理、App 首条密码创建/list、App 登录条目搜索、App 登录条目选择/编辑、App 登录条目删除/恢复、App 登录条目收藏且不改 payload、App 安全笔记/TOTP/银行卡/证件元数据收藏且不改 payload、多类型列表收藏优先排序和 Favorites Only 筛选、App 生成登录密码只填草稿不落库、App 生成选中登录密码只填编辑草稿不落库、App 安全笔记创建/搜索/编辑/删除/恢复、App TOTP 创建/搜索/编辑/删除/恢复、App 银行卡创建/编辑/删除/恢复、App 证件元数据创建/编辑/删除/恢复、App TOTP seed 生成验证码、App TOTP URI 导入填表且不落库、扫描到的 TOTP QR payload 导入填表且不落库、无效 TOTP QR 使用可读提示且不改草稿、TOTP 剩余秒数计算、后台自动锁定、inactive 隐私遮罩、自动锁定窗口、自动锁定预设、策略切换、认证 AutoFill index key 解密 Storage 索引 payload、AutoFill index key material provider 创建/复用、主 App 加密 AutoFill 索引生成、条目变更自动同步索引、主 App 写入可供 Extension 解锁填充的 AutoFill secret snapshot、条目变更同步 AutoFill credential identities、App 保存 Keychain-protected security key material 且不持久化主密码、Keychain unlock 认证后用 MDBX `security_key` 打开 remembered vault、WebDAV active vault 上传、WebDAV 恢复预览不覆盖本地 vault、WebDAV 确认恢复替换 vault 并锁定 session、WebDAV 候选备份打不开时不覆盖本地 vault、WebDAV 认证失败/完整性失败/网络断开用户提示测试；最近一次 `iPhone 17 Pro` 模拟器运行通过 53 个 XCTest，且 AutoFill Extension target 随 App 编译和嵌入通过。无签名测试使用 `CODE_SIGNING_ALLOWED=NO` 时 App Group container 会不可用并打印 `client is not entitled`，生产工厂在此情况下退回不启用 AutoFill index/secret store/provider；TOTP 相机权限和真实扫描画面仍需要签名真机验证。
- `Info.plist` 必须包含 `UILaunchScreen`，否则在 iPhone 17 Pro 模拟器上会以兼容窗口/letterbox 方式显示。
- 当前用户可见文案已完成中文化：主 App、AutoFill Extension、相机权限说明、Swift Package 基线状态和用户可读错误均已改为中文；保留英文的内容主要是技术名、系统/框架标识、bundle/service identifier、asset catalog 元数据、算法名和协议名，例如 Monica、MDBX、TOTP、WebDAV、Keychain、App Group、SHA-256、UniFFI。
- iOS AppIcon 已复用 Android launcher 图标：源文件为 `Monica for Android/app/src/main/res/drawable-nodpi/monica_launcher.png`，输出到 `Monica for iOS/App/MonicaApp/Assets.xcassets/AppIcon.appiconset/`。生成资源包含 iPhone 所需 20/29/40/60 pt 的 2x/3x PNG 和 1024 px marketing 图标；PNG 已重新生成为不透明文件，避免 App Store / Xcode 对 alpha channel 的图标约束问题。
- Xcode project 已接入 `Assets.xcassets` file reference 和 `PBXResourcesBuildPhase`，`Monica` target 会把 AppIcon asset catalog 当作资源编译；`knownRegions` 已包含 `zh-Hans`，Extension display name 当前为 `Monica 自动填充`。

## 参考链接

- Apple AuthenticationServices: https://developer.apple.com/documentation/authenticationservices
- `ASCredentialProviderViewController`: https://developer.apple.com/documentation/authenticationservices/ascredentialproviderviewcontroller
- Apple LocalAuthentication: https://developer.apple.com/documentation/localauthentication
- Apple App Groups entitlement: https://developer.apple.com/documentation/bundleresources/entitlements/com.apple.security.application-groups
- UniFFI user guide: https://mozilla.github.io/uniffi-rs/latest/tutorial/foreign_language_bindings.html
