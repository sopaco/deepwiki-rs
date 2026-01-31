use serde::{Deserialize, Serialize};

/// Target language type
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum TargetLanguage {
    #[serde(rename = "zh")]
    Chinese,
    #[serde(rename = "en")]
    English,
    #[serde(rename = "ja")]
    Japanese,
    #[serde(rename = "ko")]
    Korean,
    #[serde(rename = "de")]
    German,
    #[serde(rename = "fr")]
    French,
    #[serde(rename = "ru")]
    Russian,
    #[serde(rename = "vi")]
    Vietnamese,
}

impl Default for TargetLanguage {
    fn default() -> Self {
        Self::English
    }
}

impl std::fmt::Display for TargetLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TargetLanguage::Chinese => write!(f, "zh"),
            TargetLanguage::English => write!(f, "en"),
            TargetLanguage::Japanese => write!(f, "ja"),
            TargetLanguage::Korean => write!(f, "ko"),
            TargetLanguage::German => write!(f, "de"),
            TargetLanguage::French => write!(f, "fr"),
            TargetLanguage::Russian => write!(f, "ru"),
            TargetLanguage::Vietnamese => write!(f, "vi"),
        }
    }
}

impl std::str::FromStr for TargetLanguage {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "zh" | "chinese" | "中文" => Ok(TargetLanguage::Chinese),
            "en" | "english" | "英文" => Ok(TargetLanguage::English),
            "ja" | "japanese" | "日本語" | "日文" => Ok(TargetLanguage::Japanese),
            "ko" | "korean" | "한국어" | "韩文" => Ok(TargetLanguage::Korean),
            "de" | "german" | "deutsch" | "德文" => Ok(TargetLanguage::German),
            "fr" | "french" | "français" | "法文" => Ok(TargetLanguage::French),
            "ru" | "russian" | "русский" | "俄文" => Ok(TargetLanguage::Russian),
            "vi" | "vietnamese" | "vn" | "vietnam" => Ok(TargetLanguage::Vietnamese),
            _ => Err(format!("Unknown target language: {}", s)),
        }
    }
}

impl TargetLanguage {
    /// Get the descriptive name of the language
    pub fn display_name(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "中文",
            TargetLanguage::English => "English",
            TargetLanguage::Japanese => "日本語",
            TargetLanguage::Korean => "한국어",
            TargetLanguage::German => "Deutsch",
            TargetLanguage::French => "Français",
            TargetLanguage::Russian => "Русский",
            TargetLanguage::Vietnamese => "Tiếng Việt",
        }
    }

    /// Get the prompt instruction for the language
    pub fn prompt_instruction(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "请使用中文编写文档，确保语言表达准确、专业、易于理解。",
            TargetLanguage::English => "Please write the documentation in English, ensuring accurate, professional, and easy-to-understand language.",
            TargetLanguage::Japanese => "日本語でドキュメントを作成してください。正確で専門的で理解しやすい言語表現を心がけてください。",
            TargetLanguage::Korean => "한국어로 문서를 작성해 주세요. 정확하고 전문적이며 이해하기 쉬운 언어 표현을 사용해 주세요.",
            TargetLanguage::German => "Bitte schreiben Sie die Dokumentation auf Deutsch und stellen Sie sicher, dass die Sprache präzise, professionell und leicht verständlich ist.",
            TargetLanguage::French => "Veuillez rédiger la documentation en français, en vous assurant que le langage soit précis, professionnel et facile à comprendre.",
            TargetLanguage::Russian => "Пожалуйста, напишите документацию на русском языке, обеспечив точность, профессионализм и понятность изложения.",
            TargetLanguage::Vietnamese => "Hãy viết toàn bộ tài liệu bằng tiếng Việt tự nhiên, chính xác và dễ hiểu, sử dụng đúng thuật ngữ kỹ thuật.",
        }
    }

    /// Get directory name
    pub fn get_directory_name(&self, dir_type: &str) -> String {
        match self {
            TargetLanguage::Chinese => {
                match dir_type {
                    "deep_exploration" => "4、深入探索".to_string(),
                    _ => dir_type.to_string(),
                }
            }
            TargetLanguage::English => {
                match dir_type {
                    "deep_exploration" => "4.Deep-Exploration".to_string(),
                    _ => dir_type.to_string(),
                }
            }
            TargetLanguage::Japanese => {
                match dir_type {
                    "deep_exploration" => "4-詳細探索".to_string(),
                    _ => dir_type.to_string(),
                }
            }
            TargetLanguage::Korean => {
                match dir_type {
                    "deep_exploration" => "4-심층-탐색".to_string(),
                    _ => dir_type.to_string(),
                }
            }
            TargetLanguage::German => {
                match dir_type {
                    "deep_exploration" => "4-Tiefere-Erkundung".to_string(),
                    _ => dir_type.to_string(),
                }
            }
            TargetLanguage::French => {
                match dir_type {
                    "deep_exploration" => "4-Exploration-Approfondie".to_string(),
                    _ => dir_type.to_string(),
                }
            }
            TargetLanguage::Russian => {
                match dir_type {
                    "deep_exploration" => "4-Глубокое-Исследование".to_string(),
                    _ => dir_type.to_string(),
                }
            }
            TargetLanguage::Vietnamese => {
                match dir_type {
                    "deep_exploration" => "4-Khám-phá-chi-tiết".to_string(),
                    _ => dir_type.to_string(),
                }
            }
        }
    }

    /// Get document filename
    pub fn get_doc_filename(&self, doc_type: &str) -> String {
        match self {
            TargetLanguage::Chinese => {
                match doc_type {
                    "overview" => "1、项目概述.md".to_string(),
                    "architecture" => "2、架构概览.md".to_string(),
                    "workflow" => "3、工作流程.md".to_string(),
                    "boundary" => "5、边界调用.md".to_string(),
                    "database" => "6、数据库概览.md".to_string(),
                    _ => format!("{}.md", doc_type),
                }
            }
            TargetLanguage::English => {
                match doc_type {
                    "overview" => "1.Overview.md".to_string(),
                    "architecture" => "2.Architecture.md".to_string(),
                    "workflow" => "3.Workflow.md".to_string(),
                    "boundary" => "5.Boundary-Interfaces.md".to_string(),
                    "database" => "6.Database-Overview.md".to_string(),
                    _ => format!("{}.md", doc_type),
                }
            }
            TargetLanguage::Japanese => {
                match doc_type {
                    "overview" => "1-プロジェクト概要.md".to_string(),
                    "architecture" => "2-アーキテクチャ概要.md".to_string(),
                    "workflow" => "3-ワークフロー.md".to_string(),
                    "boundary" => "5-境界インターフェース.md".to_string(),
                    "database" => "6-データベース概要.md".to_string(),
                    _ => format!("{}.md", doc_type),
                }
            }
            TargetLanguage::Korean => {
                match doc_type {
                    "overview" => "1-프로젝트-개요.md".to_string(),
                    "architecture" => "2-아키텍처-개요.md".to_string(),
                    "workflow" => "3-워크플로우.md".to_string(),
                    "boundary" => "5-경계-인터페이스.md".to_string(),
                    "database" => "6-데이터베이스-개요.md".to_string(),
                    _ => format!("{}.md", doc_type),
                }
            }
            TargetLanguage::German => {
                match doc_type {
                    "overview" => "1-Projektübersicht.md".to_string(),
                    "architecture" => "2-Architekturübersicht.md".to_string(),
                    "workflow" => "3-Arbeitsablauf.md".to_string(),
                    "boundary" => "5-Grenzschnittstellen.md".to_string(),
                    "database" => "6-Datenbankübersicht.md".to_string(),
                    _ => format!("{}.md", doc_type),
                }
            }
            TargetLanguage::French => {
                match doc_type {
                    "overview" => "1-Aperçu-du-Projet.md".to_string(),
                    "architecture" => "2-Aperçu-de-l'Architecture.md".to_string(),
                    "workflow" => "3-Flux-de-Travail.md".to_string(),
                    "boundary" => "5-Interfaces-de-Frontière.md".to_string(),
                    "database" => "6-Aperçu-Base-de-Données.md".to_string(),
                    _ => format!("{}.md", doc_type),
                }
            }
            TargetLanguage::Russian => {
                match doc_type {
                    "overview" => "1-Обзор-Проекта.md".to_string(),
                    "architecture" => "2-Обзор-Архитектуры.md".to_string(),
                    "workflow" => "3-Рабочий-Процесс.md".to_string(),
                    "boundary" => "5-Граничные-Интерфейсы.md".to_string(),
                    "database" => "6-Обзор-Базы-Данных.md".to_string(),
                    _ => format!("{}.md", doc_type),
                }
            }
            TargetLanguage::Vietnamese => {
                match doc_type {
                    "overview" => "1-Tổng-quan-Dự-án.md".to_string(),
                    "architecture" => "2-Kiến-trúc.md".to_string(),
                    "workflow" => "3-Luồng-xử-lý.md".to_string(),
                    "boundary" => "5-Lớp-giao-tiếp-biên.md".to_string(),
                    "database" => "6-Tổng-quan-Cơ-sở-Dữ-liệu.md".to_string(),
                    _ => format!("{}.md", doc_type),
                }
            }
        }
    }

    // ===== Console Messages Translation System =====

    /// Warning: Cannot read config file, using default config
    pub fn msg_config_read_error(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "⚠️ 警告: 无法读取配置文件 {:?}，使用默认配置",
            TargetLanguage::English => "⚠️ Warning: Cannot read config file {:?}, using default config",
            TargetLanguage::Japanese => "⚠️ 警告: 設定ファイル {:?} を読み込めません、デフォルト設定を使用します",
            TargetLanguage::Korean => "⚠️ 경고: 설정 파일 {:?}을(를) 읽을 수 없습니다. 기본 설정을 사용합니다",
            TargetLanguage::German => "⚠️ Warnung: Konfigurationsdatei {:?} kann nicht gelesen werden, verwende Standardkonfiguration",
            TargetLanguage::French => "⚠️ Avertissement: Impossible de lire le fichier de configuration {:?}, utilisation de la configuration par défaut",
            TargetLanguage::Russian => "⚠️ Предупреждение: Не удается прочитать файл конфигурации {:?}, используется конфигурация по умолчанию",
            TargetLanguage::Vietnamese => "⚠️ Cảnh báo: Không thể đọc tệp cấu hình {:?}, sử dụng cấu hình mặc định",
        }
    }

    /// Warning: Unknown provider, using default provider
    pub fn msg_unknown_provider(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "⚠️ 警告: 未知的provider: {}，使用默认provider",
            TargetLanguage::English => "⚠️ Warning: Unknown provider: {}, using default provider",
            TargetLanguage::Japanese => "⚠️ 警告: 不明なプロバイダー: {}、デフォルトプロバイダーを使用します",
            TargetLanguage::Korean => "⚠️ 경고: 알 수 없는 프로바이더: {}, 기본 프로바이더를 사용합니다",
            TargetLanguage::German => "⚠️ Warnung: Unbekannter Provider: {}, verwende Standard-Provider",
            TargetLanguage::French => "⚠️ Avertissement: Fournisseur inconnu: {}, utilisation du fournisseur par défaut",
            TargetLanguage::Russian => "⚠️ Предупреждение: Неизвестный провайдер: {}, используется провайдер по умолчанию",
            TargetLanguage::Vietnamese => "⚠️ Cảnh báo: Nhà cung cấp không xác định: {}, sử dụng nhà cung cấp mặc định",
        }
    }

    /// Warning: Unknown target language, using default language (English)
    pub fn msg_unknown_language(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "⚠️ 警告: 未知的目标语言: {}，使用默认语言 (English)",
            TargetLanguage::English => "⚠️ Warning: Unknown target language: {}, using default language (English)",
            TargetLanguage::Japanese => "⚠️ 警告: 不明な対象言語: {}、デフォルト言語 (English) を使用します",
            TargetLanguage::Korean => "⚠️ 경고: 알 수 없는 대상 언어: {}, 기본 언어(English)를 사용합니다",
            TargetLanguage::German => "⚠️ Warnung: Unbekannte Zielsprache: {}, verwende Standardsprache (English)",
            TargetLanguage::French => "⚠️ Avertissement: Langue cible inconnue: {}, utilisation de la langue par défaut (English)",
            TargetLanguage::Russian => "⚠️ Предупреждение: Неизвестный целевой язык: {}, используется язык по умолчанию (English)",
            TargetLanguage::Vietnamese => "⚠️ Cảnh báo: Ngôn ngữ đích không xác định: {}, sử dụng ngôn ngữ mặc định (English)",
        }
    }

    /// Using cached AI analysis result
    pub fn msg_cache_hit(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "   ✅ 使用缓存的AI分析结果: {}",
            TargetLanguage::English => "   ✅ Using cached AI analysis result: {}",
            TargetLanguage::Japanese => "   ✅ キャッシュされたAI分析結果を使用: {}",
            TargetLanguage::Korean => "   ✅ 캐시된 AI 분석 결과 사용: {}",
            TargetLanguage::German => "   ✅ Verwende gecachtes KI-Analyseergebnis: {}",
            TargetLanguage::French => "   ✅ Utilisation du résultat d'analyse IA en cache: {}",
            TargetLanguage::Russian => "   ✅ Использование кэшированного результата AI-анализа: {}",
            TargetLanguage::Vietnamese => "   ✅ Sử dụng kết quả phân tích AI đã lưu: {}",
        }
    }

    /// Performing AI analysis
    pub fn msg_ai_analyzing(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "   🤖 正在进行AI分析: {}",
            TargetLanguage::English => "   🤖 Performing AI analysis: {}",
            TargetLanguage::Japanese => "   🤖 AI分析を実行中: {}",
            TargetLanguage::Korean => "   🤖 AI 분석 수행 중: {}",
            TargetLanguage::German => "   🤖 Führe KI-Analyse durch: {}",
            TargetLanguage::French => "   🤖 Analyse IA en cours: {}",
            TargetLanguage::Russian => "   🤖 Выполнение AI-анализа: {}",
            TargetLanguage::Vietnamese => "   🤖 Đang thực hiện phân tích AI: {}",
        }
    }

    /// Cache miss - need AI inference
    pub fn msg_cache_miss(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "   ⌛ 缓存未命中 [{}] - 需要进行AI推理",
            TargetLanguage::English => "   ⌛ Cache miss [{}] - AI inference required",
            TargetLanguage::Japanese => "   ⌛ キャッシュミス [{}] - AI推論が必要です",
            TargetLanguage::Korean => "   ⌛ 캐시 미스 [{}] - AI 추론 필요",
            TargetLanguage::German => "   ⌛ Cache-Miss [{}] - KI-Inferenz erforderlich",
            TargetLanguage::French => "   ⌛ Cache manqué [{}] - Inférence IA requise",
            TargetLanguage::Russian => "   ⌛ Промах кэша [{}] - требуется AI-вывод",
            TargetLanguage::Vietnamese => "   ⌛ Bỏ lỡ bộ nhớ cache [{}] - Cần suy luận AI",
        }
    }

    /// Cache write - result cached
    pub fn msg_cache_write(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "   💾 缓存写入 [{}] - 结果已缓存",
            TargetLanguage::English => "   💾 Cache write [{}] - Result cached",
            TargetLanguage::Japanese => "   💾 キャッシュ書き込み [{}] - 結果がキャッシュされました",
            TargetLanguage::Korean => "   💾 캐시 쓰기 [{}] - 결과 캐시됨",
            TargetLanguage::German => "   💾 Cache-Schreiben [{}] - Ergebnis gecacht",
            TargetLanguage::French => "   💾 Écriture en cache [{}] - Résultat mis en cache",
            TargetLanguage::Russian => "   💾 Запись в кэш [{}] - Результат кэширован",
            TargetLanguage::Vietnamese => "   💾 Ghi bộ nhớ cache [{}] - Kết quả đã được lưu",
        }
    }

    /// Cache error
    pub fn msg_cache_error(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "   ❌ 缓存错误 [{}]: {}",
            TargetLanguage::English => "   ❌ Cache error [{}]: {}",
            TargetLanguage::Japanese => "   ❌ キャッシュエラー [{}]: {}",
            TargetLanguage::Korean => "   ❌ 캐시 오류 [{}]: {}",
            TargetLanguage::German => "   ❌ Cache-Fehler [{}]: {}",
            TargetLanguage::French => "   ❌ Erreur de cache [{}]: {}",
            TargetLanguage::Russian => "   ❌ Ошибка кэша [{}]: {}",
            TargetLanguage::Vietnamese => "   ❌ Lỗi bộ nhớ cache [{}]: {}",
        }
    }

    /// Using cached compression result
    pub fn msg_cache_compression_hit(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "   💾 使用缓存的压缩结果 [{}]",
            TargetLanguage::English => "   💾 Using cached compression result [{}]",
            TargetLanguage::Japanese => "   💾 キャッシュされた圧縮結果を使用 [{}]",
            TargetLanguage::Korean => "   💾 캐시된 압축 결과 사용 [{}]",
            TargetLanguage::German => "   💾 Verwende gecachtes Kompressionsergebnis [{}]",
            TargetLanguage::French => "   💾 Utilisation du résultat de compression en cache [{}]",
            TargetLanguage::Russian => "   💾 Использование кэшированного результата сжатия [{}]",
            TargetLanguage::Vietnamese => "   💾 Sử dụng kết quả nén đã lưu [{}]",
        }
    }

    /// Cannot read file
    pub fn msg_cannot_read_file(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "无法读取文件: {}",
            TargetLanguage::English => "Cannot read file: {}",
            TargetLanguage::Japanese => "ファイルを読み込めません: {}",
            TargetLanguage::Korean => "파일을 읽을 수 없습니다: {}",
            TargetLanguage::German => "Datei kann nicht gelesen werden: {}",
            TargetLanguage::French => "Impossible de lire le fichier: {}",
            TargetLanguage::Russian => "Не удается прочитать файл: {}",
            TargetLanguage::Vietnamese => "Không thể đọc tệp: {}",
        }
    }

    /// AI service call error
    pub fn msg_ai_service_error(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "❌ 调用模型服务出错，尝试 {} 次均失败...{}",
            TargetLanguage::English => "❌ AI service call failed after {} attempts...{}",
            TargetLanguage::Japanese => "❌ モデルサービスの呼び出しに失敗しました。{} 回の試行に失敗...{}",
            TargetLanguage::Korean => "❌ 모델 서비스 호출 오류, {} 번의 시도 모두 실패...{}",
            TargetLanguage::German => "❌ Fehler beim Aufruf des Modell-Service, {} Versuche fehlgeschlagen...{}",
            TargetLanguage::French => "❌ Erreur d'appel du service de modèle, {} tentatives échouées...{}",
            TargetLanguage::Russian => "❌ Ошибка вызова модельного сервиса, {} попыток неудачно...{}",
            TargetLanguage::Vietnamese => "❌ Lỗi gọi dịch vụ mô hình, thất bại sau {} lần thử...{}",
        }
    }

    /// Agent type display names
    pub fn msg_agent_type(&self, agent_type: &str) -> String {
        match agent_type {
            "system_context" => match self {
                TargetLanguage::Chinese => "项目概览调研报告",
                TargetLanguage::English => "System Context Research Report",
                TargetLanguage::Japanese => "システムコンテキスト調査レポート",
                TargetLanguage::Korean => "시스템 컨텍스트 조사 보고서",
                TargetLanguage::German => "Systemkontext-Forschungsbericht",
                TargetLanguage::French => "Rapport de recherche sur le contexte système",
                TargetLanguage::Russian => "Отчет об исследовании системного контекста",
                TargetLanguage::Vietnamese => "Báo cáo nghiên cứu ngữ cảnh hệ thống",
            }.to_string(),
            "domain_modules" => match self {
                TargetLanguage::Chinese => "领域模块调研报告",
                TargetLanguage::English => "Domain Modules Research Report",
                TargetLanguage::Japanese => "ドメインモジュール調査レポート",
                TargetLanguage::Korean => "도메인 모듈 조사 보고서",
                TargetLanguage::German => "Domain-Modul-Forschungsbericht",
                TargetLanguage::French => "Rapport de recherche sur les modules de domaine",
                TargetLanguage::Russian => "Отчет об исследовании доменных модулей",
                TargetLanguage::Vietnamese => "Báo cáo nghiên cứu mô-đun miền",
            }.to_string(),
            "architecture" => match self {
                TargetLanguage::Chinese => "系统架构调研报告",
                TargetLanguage::English => "System Architecture Research Report",
                TargetLanguage::Japanese => "システムアーキテクチャ調査レポート",
                TargetLanguage::Korean => "시스템 아키텍처 조사 보고서",
                TargetLanguage::German => "Systemarchitektur-Forschungsbericht",
                TargetLanguage::French => "Rapport de recherche sur l'architecture système",
                TargetLanguage::Russian => "Отчет об исследовании системной архитектуры",
                TargetLanguage::Vietnamese => "Báo cáo nghiên cứu kiến trúc hệ thống",
            }.to_string(),
            "workflow" => match self {
                TargetLanguage::Chinese => "工作流调研报告",
                TargetLanguage::English => "Workflow Research Report",
                TargetLanguage::Japanese => "ワークフロー調査レポート",
                TargetLanguage::Korean => "워크플로우 조사 보고서",
                TargetLanguage::German => "Workflow-Forschungsbericht",
                TargetLanguage::French => "Rapport de recherche sur le flux de travail",
                TargetLanguage::Russian => "Отчет об исследовании рабочего процесса",
                TargetLanguage::Vietnamese => "Báo cáo nghiên cứu quy trình làm việc",
            }.to_string(),
            "key_modules" => match self {
                TargetLanguage::Chinese => "核心模块与组件调研报告",
                TargetLanguage::English => "Key Modules and Components Research Report",
                TargetLanguage::Japanese => "主要モジュールとコンポーネント調査レポート",
                TargetLanguage::Korean => "핵심 모듈 및 구성 요소 조사 보고서",
                TargetLanguage::German => "Schlüsselmodul- und Komponenten-Forschungsbericht",
                TargetLanguage::French => "Rapport de recherche sur les modules et composants clés",
                TargetLanguage::Russian => "Отчет об исследовании ключевых модулей и компонентов",
                TargetLanguage::Vietnamese => "Báo cáo nghiên cứu mô-đun và thành phần chính",
            }.to_string(),
            "boundary" => match self {
                TargetLanguage::Chinese => "边界接口调研报告",
                TargetLanguage::English => "Boundary Interface Research Report",
                TargetLanguage::Japanese => "境界インターフェース調査レポート",
                TargetLanguage::Korean => "경계 인터페이스 조사 보고서",
                TargetLanguage::German => "Grenzschnittstellenforschungsbericht",
                TargetLanguage::French => "Rapport de recherche sur les interfaces de frontière",
                TargetLanguage::Russian => "Отчет об исследовании граничных интерфейсов",
                TargetLanguage::Vietnamese => "Báo cáo nghiên cứu giao diện biên",
            }.to_string(),
            "database" => match self {
                TargetLanguage::Chinese => "数据库概览调研报告",
                TargetLanguage::English => "Database Overview Research Report",
                TargetLanguage::Japanese => "データベース概要調査レポート",
                TargetLanguage::Korean => "데이터베이스 개요 조사 보고서",
                TargetLanguage::German => "Datenbankübersicht-Forschungsbericht",
                TargetLanguage::French => "Rapport de recherche sur l'aperçu de la base de données",
                TargetLanguage::Russian => "Отчет об исследовании обзора базы данных",
                TargetLanguage::Vietnamese => "Báo cáo nghiên cứu tổng quan cơ sở dữ liệu",
            }.to_string(),
            _ => agent_type.to_string(),
        }
    }

    /// Warning: Document content not found
    pub fn msg_doc_not_found(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "⚠️ 警告: 未找到文档内容，键: {}",
            TargetLanguage::English => "⚠️ Warning: Document content not found, key: {}",
            TargetLanguage::Japanese => "⚠️ 警告: ドキュメントコンテンツが見つかりません、キー: {}",
            TargetLanguage::Korean => "⚠️ 경고: 문서 내용을 찾을 수 없습니다, 키: {}",
            TargetLanguage::German => "⚠️ Warnung: Dokumentinhalt nicht gefunden, Schlüssel: {}",
            TargetLanguage::French => "⚠️ Avertissement: Contenu du document introuvable, clé: {}",
            TargetLanguage::Russian => "⚠️ Предупреждение: Содержимое документа не найдено, ключ: {}",
            TargetLanguage::Vietnamese => "⚠️ Cảnh báo: Không tìm thấy nội dung tài liệu, khóa: {}",
        }
    }

    /// Mermaid fixer error
    pub fn msg_mermaid_error(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "⚠️ mermaid图表修复过程中出现错误: {}",
            TargetLanguage::English => "⚠️ Error occurred during mermaid diagram repair: {}",
            TargetLanguage::Japanese => "⚠️ mermaidダイアグラムの修復中にエラーが発生しました: {}",
            TargetLanguage::Korean => "⚠️ mermaid 다이어그램 복구 중 오류 발생: {}",
            TargetLanguage::German => "⚠️ Fehler während der Mermaid-Diagrammreparatur aufgetreten: {}",
            TargetLanguage::French => "⚠️ Erreur survenue lors de la réparation du diagramme mermaid: {}",
            TargetLanguage::Russian => "⚠️ Ошибка при восстановлении диаграммы mermaid: {}",
            TargetLanguage::Vietnamese => "⚠️ Lỗi xảy ra trong quá trình sửa chữa sơ đồ mermaid: {}",
        }
    }

    /// Mermaid fixer not installed
    pub fn msg_mermaid_not_installed(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "⚠️ 警告: mermaid-fixer 未安装或不可用，跳过mermaid图表修复",
            TargetLanguage::English => "⚠️ Warning: mermaid-fixer not installed or unavailable, skipping mermaid diagram repair",
            TargetLanguage::Japanese => "⚠️ 警告: mermaid-fixer がインストールされていないか利用できません、mermaidダイアグラムの修復をスキップします",
            TargetLanguage::Korean => "⚠️ 경고: mermaid-fixer가 설치되지 않았거나 사용할 수 없습니다. mermaid 다이어그램 복구를 건너뜁니다",
            TargetLanguage::German => "⚠️ Warnung: mermaid-fixer nicht installiert oder nicht verfügbar, überspringe Mermaid-Diagrammreparatur",
            TargetLanguage::French => "⚠️ Avertissement: mermaid-fixer non installé ou indisponible, réparation du diagramme mermaid ignorée",
            TargetLanguage::Russian => "⚠️ Предупреждение: mermaid-fixer не установлен или недоступен, пропуск восстановления диаграммы mermaid",
            TargetLanguage::Vietnamese => "⚠️ Cảnh báo: mermaid-fixer chưa cài đặt hoặc không khả dụng, bỏ qua sửa chữa sơ đồ mermaid",
        }
    }

    /// Summary reasoning failed
    pub fn msg_summary_reasoning_failed(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "⚠️  总结推理失败，返回原始部分结果...{}",
            TargetLanguage::English => "⚠️  Summary reasoning failed, returning original partial result...{}",
            TargetLanguage::Japanese => "⚠️  要約推論に失敗しました、元の部分的な結果を返します...{}",
            TargetLanguage::Korean => "⚠️  요약 추론 실패, 원래 부분 결과 반환...{}",
            TargetLanguage::German => "⚠️  Zusammenfassungs-Reasoning fehlgeschlagen, gebe ursprüngliches Teilergebnis zurück...{}",
            TargetLanguage::French => "⚠️  Échec du raisonnement de résumé, retour du résultat partiel d'origine...{}",
            TargetLanguage::Russian => "⚠️  Сбой суммирования, возврат исходного частичного результата...{}",
            TargetLanguage::Vietnamese => "⚠️  Suy luận tóm tắt thất bại, trả về kết quả một phần ban đầu...{}",
        }
    }

    /// Max iterations reached
    pub fn msg_max_iterations(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "   ⚠️ 达到最大迭代次数 ({}), 触发中断",
            TargetLanguage::English => "   ⚠️ Maximum iterations reached ({}), interrupting",
            TargetLanguage::Japanese => "   ⚠️ 最大反復回数に達しました ({})、中断します",
            TargetLanguage::Korean => "   ⚠️ 최대 반복 횟수 도달 ({}), 중단",
            TargetLanguage::German => "   ⚠️ Maximale Iterationen erreicht ({}), unterbreche",
            TargetLanguage::French => "   ⚠️ Nombre maximum d'itérations atteint ({}), interruption",
            TargetLanguage::Russian => "   ⚠️ Достигнуто максимальное количество итераций ({}), прерывание",
            TargetLanguage::Vietnamese => "   ⚠️ Đạt số lần lặp tối đa ({}), đang ngắt",
        }
    }

    /// Domain analysis failed
    pub fn msg_domain_analysis_failed(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "⚠️ 领域模块分析：{} 分析失败: {}",
            TargetLanguage::English => "⚠️ Domain module analysis: {} analysis failed: {}",
            TargetLanguage::Japanese => "⚠️ ドメインモジュール分析：{} の分析に失敗しました: {}",
            TargetLanguage::Korean => "⚠️ 도메인 모듈 분석: {} 분석 실패: {}",
            TargetLanguage::German => "⚠️ Domain-Modul-Analyse: {} Analyse fehlgeschlagen: {}",
            TargetLanguage::French => "⚠️ Analyse du module de domaine: échec de l'analyse de {}: {}",
            TargetLanguage::Russian => "⚠️ Анализ доменного модуля: анализ {} не удался: {}",
            TargetLanguage::Vietnamese => "⚠️ Phân tích mô-đun miền: phân tích {} thất bại: {}",
        }
    }

    /// No code path for domain
    pub fn msg_no_code_path_for_domain(&self) -> &'static str {
        match self {
            TargetLanguage::Chinese => "⚠️ 领域'{}'没有关联的代码路径",
            TargetLanguage::English => "⚠️ Domain '{}' has no associated code paths",
            TargetLanguage::Japanese => "⚠️ ドメイン'{}'に関連するコードパスがありません",
            TargetLanguage::Korean => "⚠️ 도메인 '{}'에 연결된 코드 경로가 없습니다",
            TargetLanguage::German => "⚠️ Domain '{}' hat keine zugeordneten Code-Pfade",
            TargetLanguage::French => "⚠️ Le domaine '{}' n'a pas de chemins de code associés",
            TargetLanguage::Russian => "⚠️ Домен '{}' не имеет связанных путей кода",
            TargetLanguage::Vietnamese => "⚠️ Miền '{}' không có đường dẫn mã liên kết",
        }
    }
}