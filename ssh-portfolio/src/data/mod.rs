//! Portfolio data for Ali Turan (Alttre) — mirrored from `alttre.is-a.dev`
//! (`app.js`, `competitions.js`, `index.html`, `/tr/`, `/about/`).
//!
//! Every user-visible string is bilingual ([`Bi`]) so the TUI can flip between
//! English and Turkish with a single keypress.

/// UI language.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    #[default]
    En,
    Tr,
}

impl Lang {
    pub fn toggle(self) -> Self {
        match self {
            Lang::En => Lang::Tr,
            Lang::Tr => Lang::En,
        }
    }

    pub fn code(self) -> &'static str {
        match self {
            Lang::En => "EN",
            Lang::Tr => "TR",
        }
    }
}

/// A bilingual static string.
#[derive(Debug, Clone, Copy)]
pub struct Bi {
    pub en: &'static str,
    pub tr: &'static str,
}

impl Bi {
    pub const fn new(en: &'static str, tr: &'static str) -> Self {
        Self { en, tr }
    }

    pub fn get(&self, lang: Lang) -> &'static str {
        match lang {
            Lang::En => self.en,
            Lang::Tr => self.tr,
        }
    }
}

const fn bi(en: &'static str, tr: &'static str) -> Bi {
    Bi::new(en, tr)
}

// ───────────────────────────── Identity / Intro ─────────────────────────────

pub const NAME: &str = "Ali Turan";
pub const HANDLE: &str = "@Alttre";
pub const GITHUB_HANDLE: &str = "@Alttrest";
pub const SITE: &str = "alttre.is-a.dev";
pub const STAMP: &str = "ALTTRE // 2026";

pub const ROLE: Bi = bi(
    "Information Technology student · Software developer",
    "Bilişim Teknolojileri öğrencisi · Yazılım geliştirici",
);

/// ASCII robot mascot from `index.html`.
pub const MASCOT: [&str; 6] = [
    "       .-=========-.",
    "      /  .-^^^-.  \\",
    "     |  /  ◉ ◉  \\  |",
    "     | |   ___   | |",
    "      \\  \\_____/  /",
    "       '----------'",
];

pub const INTRO_LINES: [Bi; 5] = [
    bi(
        "Hello, I'm Ali Turan (Alttre).",
        "Merhaba, ben Ali Turan (Alttre).",
    ),
    bi(
        "I'm an Information Technology student and software developer.",
        "Bilişim teknolojileri öğrencisi ve yazılım geliştiricisiyim.",
    ),
    bi(
        "I build web systems from scratch, custom embedded hardware, and AI agents.",
        "Web sistemlerini sıfırdan, özel gömülü donanımları ve yapay zekâ ajanlarını geliştiriyorum.",
    ),
    bi(
        "I am also the co-founder of Nebula Team.",
        "Aynı zamanda Nebula Team'in kurucu ortaklarından biriyim.",
    ),
    bi(
        "Ready to explore? Scroll down or click Explore...",
        "Keşfetmeye hazır mısın? Aşağı kaydır veya Keşfet'e tıkla...",
    ),
];

pub const INTRO_SUMMARY: Bi = bi(
    "Ali Turan is a developer building web apps, AI agents, robotics and hardware projects under the name Alttre. Explore his selected work and engineering notes.",
    "Alttre, Ali Turan'ın geliştirici adı: web uygulamaları, yapay zekâ ajanları, robotik ve donanım projeleri. Seçili işlerini ve mühendislik notlarını keşfet.",
);

// ────────────────────────────────── About ──────────────────────────────────

pub const ABOUT_BIO: Bi = bi(
    "I'm Ali Turan. Alttre is the developer name I use for my software and hardware projects. I'm an Information Technology student at a Vocational and Technical High School in Türkiye, working on full-stack and frontend web systems, Python services, autonomous AI agents and robotics. I'm looking for software developer internships and opportunities in Istanbul.",
    "Ben Ali Turan. Alttre, yazılım ve donanım projelerimde kullandığım geliştirici adı. Türkiye'de bir Mesleki ve Teknik Anadolu Lisesi'nde bilişim teknolojileri öğrencisiyim; frontend arayüzler, Python tabanlı servisler, otonom yapay zekâ ajanları ve robotik üzerinde çalışıyorum. İstanbul'da yazılım geliştirici stajı ve fırsatları arıyorum.",
);

pub const NEBULA: Bi = bi(
    "Co-founder of Nebula Team — a small crew shipping software, AI agents and robots together.",
    "Nebula Team'in kurucu ortaklarından biriyim — yazılım, yapay zekâ ajanları ve robotları birlikte üreten küçük bir ekip.",
);

pub const PHILOSOPHY: [(Bi, Bi); 2] = [
    (
        bi("From-Scratch Development", "Sıfırdan Geliştirme"),
        bi(
            "Native JS, TS and Python core architectures over heavy builders and low-code drag-and-drop templates.",
            "Ağır site oluşturucular ve low-code şablonlar yerine yerel JS, TS ve Python çekirdek mimarileri.",
        ),
    ),
    (
        bi("Visual Mechanics", "Görsel Mekanikler"),
        bi(
            "Lightweight CSS 3D transforms and wheel-event layouts instead of bloating pages with heavy third-party graphic engines.",
            "Sayfaları ağır üçüncü parti grafik motorlarıyla şişirmek yerine hafif CSS 3D dönüşümleri ve tekerlek olaylarıyla kurulan düzenler.",
        ),
    ),
];

// ───────────────────────────────── Projects ─────────────────────────────────

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Ai,
    Web,
    RoboticsDesktop,
}

impl Group {
    pub fn label(self, lang: Lang) -> &'static str {
        match (self, lang) {
            (Group::Ai, Lang::En) => "AI",
            (Group::Ai, Lang::Tr) => "Yapay Zekâ",
            (Group::Web, _) => "Web",
            (Group::RoboticsDesktop, Lang::En) => "Robotics & Desktop",
            (Group::RoboticsDesktop, Lang::Tr) => "Robotik & Masaüstü",
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Project {
    pub emoji: &'static str,
    pub title: &'static str,
    pub category: &'static str,
    pub tag: &'static str,
    pub group: Group,
    pub short: Bi,
    pub full: Bi,
    pub stack: &'static [&'static str],
    pub github: &'static str,
    pub devpost: Option<&'static str>,
    pub live: Option<&'static str>,
}

pub static PROJECTS: [Project; 13] = [
    Project {
        emoji: "📱",
        title: "NexusAI-PhoneAgent",
        category: "Mobile AI",
        tag: "New / Active",
        group: Group::Ai,
        short: bi(
            "React Native and Expo based mobile AI assistant.",
            "React Native ve Expo tabanlı mobil yapay zekâ asistanı.",
        ),
        full: bi(
            "Smartphone AI assistant coordinating on-device operations with background LLM integration over JSON-RPC.",
            "Cihaz üzerindeki işlemleri koordine eden, arka planda JSON-RPC üzerinden LLM entegrasyonu sağlayan akıllı telefon yapay zekâ asistanı.",
        ),
        stack: &["React Native", "Expo", "TypeScript", "LLM APIs", "JSON-RPC"],
        github: "https://github.com/Alttrest/NexusAI-PhoneAgent",
        devpost: Some("https://devpost.com/software/nexusai-phoneagent"),
        live: None,
    },
    Project {
        emoji: "🤖",
        title: "AI-POST Ecosystem",
        category: "AI Automation",
        tag: "Autonomous",
        group: Group::Ai,
        short: bi(
            "AI-powered social media automation, posting, and management tool suite.",
            "Yapay zekâ destekli sosyal medya otomasyonu, paylaşım ve yönetim araç seti.",
        ),
        full: bi(
            "Autonomous agent ecosystem managing social content planning, image/text generation, trend analysis, and auto-posting end-to-end.",
            "Sosyal içerik planlama, görsel/metin üretimi, trend analizi ve otomatik paylaşımı uçtan uca yöneten otonom ajan ekosistemi.",
        ),
        stack: &["Python", "Flask", "Gemini API", "Playwright", "Docker"],
        github: "https://github.com/Alttrest/AI-POST",
        devpost: Some("https://devpost.com/software/ai-post-ecosystem"),
        live: None,
    },
    Project {
        emoji: "🧠",
        title: "NexusAI-Agent",
        category: "Desktop AI",
        tag: "Advanced",
        group: Group::Ai,
        short: bi(
            "Electron and React based desktop AI operator (Flux-Agent series).",
            "Electron ve React tabanlı masaüstü yapay zekâ operatörü (Flux-Agent serisi).",
        ),
        full: bi(
            "Local/cloud desktop AI assistant using ReAct (Reasoning and Acting) architecture to manage files, run terminal commands, and browse the web.",
            "ReAct (Reasoning and Acting) mimarisiyle dosyaları yöneten, terminal komutları çalıştıran ve web'de gezinen yerel/bulut masaüstü yapay zekâ asistanı.",
        ),
        stack: &["Electron", "React", "Node.js", "Python", "ReAct Framework"],
        github: "https://github.com/Alttrest/NexusAI-Agent",
        devpost: None,
        live: None,
    },
    Project {
        emoji: "🔍",
        title: "Ven-s-Finder",
        category: "Cyber Intel & OSINT",
        tag: "Fast",
        group: Group::Ai,
        short: bi(
            "Python-based person search engine using Groq AI and DuckDuckGo search API.",
            "Groq AI ve DuckDuckGo arama API'sini kullanan Python tabanlı kişi arama motoru.",
        ),
        full: bi(
            "Query-driven OSINT tool searching public web data via DuckDuckGo and synthesizing structured profiles/reports with Groq AI.",
            "DuckDuckGo ile herkese açık web verilerini tarayan ve Groq AI ile yapılandırılmış profil/rapor üreten sorgu odaklı OSINT aracı.",
        ),
        stack: &["Python", "Groq AI", "DuckDuckGo API", "BeautifulSoup"],
        github: "https://github.com/Alttrest/Ven-s-Finder",
        devpost: None,
        live: None,
    },
    Project {
        emoji: "💬",
        title: "FBMTAL Chat & AlttreHub",
        category: "Real-Time Comms",
        tag: "WebSockets",
        group: Group::Web,
        short: bi(
            "WebSockets, Socket.io, and Flask based real-time chat infrastructure.",
            "WebSockets, Socket.io ve Flask tabanlı gerçek zamanlı sohbet altyapısı.",
        ),
        full: bi(
            "Instant messaging platform for local school networks with rooms, user accounts, and real-time push notifications on Flask + Socket.io.",
            "Yerel okul ağları için odalar, kullanıcı hesapları ve gerçek zamanlı anlık bildirimler sunan Flask + Socket.io tabanlı mesajlaşma platformu.",
        ),
        stack: &["Flask", "Socket.io", "WebSockets", "JavaScript", "HTML/CSS"],
        github: "https://github.com/Alttrest",
        devpost: None,
        live: None,
    },
    Project {
        emoji: "⚙️",
        title: "Mini Sumo Robot",
        category: "Robotics & Embedded",
        tag: "Award Winning 🏆",
        group: Group::RoboticsDesktop,
        short: bi(
            "Autonomous mini sumo robot project, winner of the Fibonacci Robot Cup Jury Special Award.",
            "Fibonacci Robot Kupası Jüri Özel Ödülü'nü kazanan otonom mini sumo robot projesi.",
        ),
        full: bi(
            "C++/Arduino autonomous mini sumo robot processing IR/distance sensors + IMU hundreds of times per second (sensor fusion) for optimal motor torque and push tactics.",
            "IR/mesafe sensörleri ve IMU verisini saniyede yüzlerce kez işleyerek (sensör füzyonu) en uygun motor torku ve itme taktiğini seçen C++/Arduino otonom mini sumo robot.",
        ),
        stack: &["C/C++", "Arduino", "Sensor Fusion", "Motor Drivers", "Li-Po Power"],
        github: "https://github.com/Alttrest",
        devpost: Some("https://devpost.com/Alttrest"),
        live: None,
    },
    Project {
        emoji: "📰",
        title: "Hype-NEWS",
        category: "AI Media",
        tag: "Dynamic",
        group: Group::Ai,
        short: bi(
            "AI-powered interactive news station platform.",
            "Yapay zekâ destekli interaktif haber istasyonu platformu.",
        ),
        full: bi(
            "Parses global RSS/news feeds and uses AI to generate summarized, interactive briefings (with TTS & custom tone rewriting).",
            "Global RSS/haber akışlarını ayrıştırır ve yapay zekâ ile özetlenmiş, interaktif bültenler üretir (TTS ve özel ton yeniden yazımıyla).",
        ),
        stack: &["JavaScript", "Flask", "Gemini API", "HTML5", "CSS Grid"],
        github: "https://github.com/Alttrest/Hype-NEWS",
        devpost: None,
        live: None,
    },
    Project {
        emoji: "🛠️",
        title: "StartupCMD",
        category: "Desktop Utility",
        tag: "New",
        group: Group::RoboticsDesktop,
        short: bi(
            "Electron application to list, document, and manage Windows startup processes.",
            "Windows başlangıç süreçlerini listeleyen, belgeleyen ve yöneten Electron uygulaması.",
        ),
        full: bi(
            "Scans Windows startup commands/scripts/apps, explains impacts with AI, and manages startup behavior via PowerShell API.",
            "Windows başlangıç komutlarını/betiklerini/uygulamalarını tarar, etkilerini yapay zekâ ile açıklar ve PowerShell API ile başlangıç davranışını yönetir.",
        ),
        stack: &["Electron", "JavaScript", "HTML/CSS", "PowerShell API"],
        github: "https://github.com/Alttrest/startupcmd1",
        devpost: None,
        live: None,
    },
    Project {
        emoji: "🌐",
        title: "Web OS",
        category: "Web Application",
        tag: "Dynamic",
        group: Group::Web,
        short: bi(
            "Browser-based autonomous web operating system simulator written in JavaScript.",
            "JavaScript ile yazılmış, tarayıcı tabanlı otonom web işletim sistemi simülatörü.",
        ),
        full: bi(
            "Browser desktop OS simulator with draggable windows, terminal utilities, web apps, and virtual filesystem.",
            "Sürüklenebilir pencereler, terminal araçları, web uygulamaları ve sanal dosya sistemi sunan tarayıcı masaüstü işletim sistemi simülatörü.",
        ),
        stack: &["JavaScript", "HTML5", "CSS3", "Vite"],
        github: "https://github.com/Alttrest/web-os",
        devpost: None,
        live: None,
    },
    Project {
        emoji: "✨",
        title: "scroll-animation",
        category: "Web Animation",
        tag: "Visual",
        group: Group::Web,
        short: bi(
            "High-performance web scroll animation library reactive to viewport scroll inputs.",
            "Görünüm alanı kaydırma girdilerine tepki veren yüksek performanslı web kaydırma animasyon kütüphanesi.",
        ),
        full: bi(
            "Lightweight scroll-driven layout engine built with CSS 3D transforms instead of heavy JS physics libraries.",
            "Ağır JS fizik kütüphaneleri yerine CSS 3D dönüşümleriyle kurulmuş hafif, kaydırma güdümlü düzen motoru.",
        ),
        stack: &["JavaScript", "HTML5", "CSS3", "Scroll Handling"],
        github: "https://github.com/Alttrest/scroll-animation",
        devpost: None,
        live: None,
    },
    Project {
        emoji: "🏗️",
        title: "Sefa Engineering",
        category: "Corporate Showcase",
        tag: "Completed",
        group: Group::Web,
        short: bi(
            "Modern, minimalist corporate website designed for Sefa Mühendislik.",
            "Sefa Mühendislik için tasarlanmış modern, minimalist kurumsal web sitesi.",
        ),
        full: bi(
            "Fast, SEO-friendly, responsive corporate showcase site built from scratch.",
            "Sıfırdan geliştirilmiş hızlı, SEO dostu ve duyarlı kurumsal vitrin sitesi.",
        ),
        stack: &["HTML5", "CSS3", "JavaScript", "Responsive Design"],
        github: "https://github.com/Alttrest",
        devpost: None,
        live: Some("https://sefamuhendislik.com"),
    },
    Project {
        emoji: "🛋️",
        title: "ar-furniture-viewer",
        category: "Augmented Reality (AR)",
        tag: "New / Public",
        group: Group::Web,
        short: bi(
            "Immersive WebXR furniture visualizer to view 3D models in your space before buying.",
            "Satın almadan önce 3D modelleri kendi mekânında görmeni sağlayan sürükleyici WebXR mobilya görselleştirici.",
        ),
        full: bi(
            "Web-based AR (WebXR) app placing real-scale 3D furniture models inside rooms prior to purchase. Also listed as AlttreVision.",
            "Satın almadan önce gerçek ölçekli 3D mobilya modellerini odalara yerleştiren web tabanlı AR (WebXR) uygulaması. AlttreVision adıyla da listelenir.",
        ),
        stack: &["Three.js", "WebXR", "Model-Viewer", "JavaScript", "HTML/CSS"],
        github: "https://github.com/Alttrest/ar-furniture-viewer",
        devpost: None,
        live: Some("https://3d-furniture-viewer.vercel.app"),
    },
    Project {
        emoji: "💼",
        title: "İş-Bul (Job-Finder)",
        category: "Jobs & Careers",
        tag: "Istanbul · Active",
        group: Group::Web,
        short: bi(
            "Modern job matching and career tracker application for job seekers.",
            "İş arayanlar için geliştirilmiş modern iş eşleştirme ve kariyer takip uygulaması.",
        ),
        full: bi(
            "Istanbul-focused job matching and tracker board connecting seekers and recruiters with smart filters.",
            "İş arayanları ve işverenleri akıllı filtrelerle buluşturan, İstanbul odaklı iş eşleştirme ve takip panosu.",
        ),
        stack: &["JavaScript", "HTML5", "CSS3", "Vercel"],
        github: "https://github.com/Alttrest",
        devpost: None,
        live: Some("https://is-bul.vercel.app"),
    },
];

/// Project filter chips: `None` = All.
pub const PROJECT_FILTERS: [Option<Group>; 4] = [
    None,
    Some(Group::Ai),
    Some(Group::Web),
    Some(Group::RoboticsDesktop),
];

pub fn filter_label(filter: Option<Group>, lang: Lang) -> &'static str {
    match (filter, lang) {
        (None, Lang::En) => "All",
        (None, Lang::Tr) => "Tümü",
        (Some(g), l) => g.label(l),
    }
}

/// Indices into [`PROJECTS`] that match `filter`.
pub fn filtered_projects(filter: Option<Group>) -> Vec<usize> {
    PROJECTS
        .iter()
        .enumerate()
        .filter(|(_, p)| filter.is_none_or(|g| p.group == g))
        .map(|(i, _)| i)
        .collect()
}

// ────────────────────────────────── Skills ──────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Skill {
    pub name: &'static str,
    /// Focus / day-to-day usage on a 1..=10 scale (visual bar only).
    pub focus: u8,
    pub note: Bi,
}

#[derive(Debug, Clone, Copy)]
pub struct SkillCategory {
    pub name: Bi,
    pub skills: &'static [Skill],
}

const fn sk(name: &'static str, focus: u8, note: Bi) -> Skill {
    Skill { name, focus, note }
}

pub static SKILL_CATEGORIES: [SkillCategory; 5] = [
    SkillCategory {
        name: bi("Programming Languages", "Programlama Dilleri"),
        skills: &[
            sk("Python", 9, bi("Flask services, agents, OSINT tooling", "Flask servisleri, ajanlar, OSINT araçları")),
            sk("C++", 8, bi("Arduino firmware, sensor fusion", "Arduino yazılımı, sensör füzyonu")),
            sk("JavaScript", 9, bi("From-scratch web, Node, Electron", "Sıfırdan web, Node, Electron")),
            sk("HTML5 / CSS3", 8, bi("3D transforms, responsive layouts", "3D dönüşümler, duyarlı tasarım")),
            sk("Bash / Zsh", 6, bi("Shell workflow, dotfiles, automation", "Kabuk akışı, dotfile, otomasyon")),
        ],
    },
    SkillCategory {
        name: bi("AI (Yapay Zeka)", "Yapay Zekâ (AI)"),
        skills: &[
            sk("LLM API", 8, bi("Gemini, Groq and cloud LLM endpoints", "Gemini, Groq ve bulut LLM uçları")),
            sk("Prompt Engineering", 8, bi("Reliable prompts, tone rewriting", "Güvenilir istemler, ton yeniden yazımı")),
            sk("ReAct Agents", 8, bi("Reason → Act loops for autonomous agents", "Otonom ajanlar için Düşün → Uygula döngüsü")),
            sk("Playwright", 6, bi("Browser automation for agents", "Ajanlar için tarayıcı otomasyonu")),
        ],
    },
    SkillCategory {
        name: bi("Framework & Server", "Framework & Sunucu"),
        skills: &[
            sk("Flask", 8, bi("REST + real-time Python backends", "REST ve gerçek zamanlı Python sunucuları")),
            sk("Socket.io / WebSockets", 7, bi("Live chat, push notifications", "Canlı sohbet, anlık bildirim")),
            sk("Expo / React Native", 7, bi("Mobile AI assistant client", "Mobil yapay zekâ asistanı istemcisi")),
            sk("Node.js", 7, bi("Electron desktop apps, tooling", "Electron masaüstü uygulamaları, araçlar")),
        ],
    },
    SkillCategory {
        name: bi("Hardware & Lab", "Donanım & Laboratuvar"),
        skills: &[
            sk("Arduino", 8, bi("Sumo robot brain, sensors, motors", "Sumo robot beyni, sensörler, motorlar")),
            sk("Pixhawk / ArduPilot", 5, bi("Flight-controller experiments", "Uçuş kontrolcüsü denemeleri")),
            sk("Circuit Design", 7, bi("Sensor boards, motor driver wiring", "Sensör kartları, motor sürücü bağlantıları")),
            sk("Soldering", 7, bi("Robot harnesses and prototypes", "Robot kablo demetleri ve prototipler")),
            sk("Li-Po & Type-C", 7, bi("2S Li-Po power, Type-C charging", "2S Li-Po güç, Type-C şarj")),
        ],
    },
    SkillCategory {
        name: bi("Systems & Utilities", "Sistem & Araçlar"),
        skills: &[
            sk("Git / GitHub", 8, bi("Daily version control, dotfile repos", "Günlük sürüm kontrolü, dotfile depoları")),
            sk("Linux (Hyprland)", 8, bi("Arch + Hyprland, Kitty, Zsh, Starship", "Arch + Hyprland, Kitty, Zsh, Starship")),
            sk("Docker", 6, bi("Containerized agent ecosystems", "Konteynerleştirilmiş ajan ekosistemleri")),
            sk("Zyxel Nebula", 6, bi("Cloud network management", "Bulut ağ yönetimi")),
        ],
    },
];

/// Cross-links between skills (`skillLinksData` in `app.js`).
pub static SKILL_LINKS: [(&str, &str); 10] = [
    ("JavaScript", "Node.js"),
    ("JavaScript", "Expo / React Native"),
    ("Python", "Flask"),
    ("Python", "ReAct Agents"),
    ("C++", "Arduino"),
    ("C++", "Pixhawk / ArduPilot"),
    ("LLM API", "ReAct Agents"),
    ("Arduino", "Circuit Design"),
    ("Circuit Design", "Soldering"),
    ("Linux (Hyprland)", "Bash / Zsh"),
];

pub fn skill_count() -> usize {
    SKILL_CATEGORIES.iter().map(|c| c.skills.len()).sum()
}

/// Names linked to `skill`.
pub fn linked_skills(skill: &str) -> Vec<&'static str> {
    SKILL_LINKS
        .iter()
        .filter_map(|(a, b)| {
            if *a == skill {
                Some(*b)
            } else if *b == skill {
                Some(*a)
            } else {
                None
            }
        })
        .collect()
}

// ───────────────────────────── Competitions ─────────────────────────────

pub const HACKATHON_COUNT: usize = 4;
pub const AWARD_COUNT: usize = 1;

pub const COMP_KICKER: &str = "SAHADAN NOTLAR / 01—03";
pub const COMP_TITLE: Bi = bi("Competitions.", "Yarışmalar.");
pub const COMP_TAGLINE: Bi = bi(
    "From idea to robot — my competition journey.",
    "Fikirden robota, yarışma yolculuğum.",
);

pub const COMP_BANNER: [Bi; 3] = [
    bi("⚡ 36h Sprint", "⚡ 36 Saat Sprint"),
    bi("🏆 Fibonacci Jury Special Award", "🏆 Özel Ödül (Fibonacci Robot Kupası)"),
    bi("🧠 ReAct AI (Autonomous Agents)", "🧠 ReAct AI (Otonom Ajanlar)"),
];

#[derive(Debug, Clone, Copy)]
pub struct Competition {
    pub number: &'static str,
    pub name: &'static str,
    pub year: &'static str,
    pub kind: Bi,
    pub glyph: &'static str,
    /// Lanyard tone (RGB).
    pub tone: (u8, u8, u8),
    pub id_code: &'static str,
    pub blurb: Bi,
    pub award: Option<Bi>,
    pub back: &'static [Bi],
}

pub static COMPETITIONS: [Competition; 3] = [
    Competition {
        number: "HACKATHON / 01",
        name: "ALKÜ Hackathon",
        year: "2025",
        kind: bi("Hackathon", "Hackathon"),
        glyph: "✳",
        tone: (0xB4, 0x47, 0x2B),
        id_code: "ALT-2025-01",
        blurb: bi(
            "36 hours, one team, one autonomous AI agent built from zero.",
            "36 saat, tek takım, sıfırdan kurulan tek bir otonom yapay zekâ ajanı.",
        ),
        award: None,
        back: &[
            bi("⏱  36h non-stop sprint", "⏱  36 saat kesintisiz sprint"),
            bi("🧭 6h plan · 20h build · 10h debug & demo", "🧭 6s plan · 20s geliştirme · 10s hata ayıklama ve demo"),
            bi("🧠 ReAct (Reasoning + Acting) agent loop", "🧠 ReAct (Düşün + Uygula) ajan döngüsü"),
            bi("🛠  Python · Flask · Gemini API · Vanilla JS", "🛠  Python · Flask · Gemini API · Vanilla JS"),
            bi("🎤 Live demo: web research → summary → report", "🎤 Canlı demo: web araştırması → özet → rapor"),
            bi("📌 Added to my Devpost hackathon portfolio", "📌 Devpost hackathon portföyüme eklendi"),
        ],
    },
    Competition {
        number: "ROBOTİK / 02",
        name: "Fibonacci Robot Olimpiyatları",
        year: "2025",
        kind: bi("Robotics", "Robotik"),
        glyph: "✦",
        tone: (0xD4, 0x7A, 0x3B),
        id_code: "ALT-2025-02",
        blurb: bi(
            "Mini Sumo — decisions made faster than the opponent can push.",
            "Mini Sumo — rakip itmeden önce karar veren robot.",
        ),
        award: Some(bi("Jury Special Award", "Jüri Özel Ödülü")),
        back: &[
            bi("🏆 Jury Special Award — 5 of 7 matches won", "🏆 Jüri Özel Ödülü — 7 maçın 5'i kazanıldı"),
            bi("📏 4× QTR line sensors (ring edge)", "📏 4× QTR çizgi sensörü (ring kenarı)"),
            bi("📡 3× Sharp IR distance (front/left/right)", "📡 3× Sharp IR mesafe (ön/sol/sağ)"),
            bi("🧭 1× MPU6050 IMU (heading & accel)", "🧭 1× MPU6050 IMU (yön ve ivme)"),
            bi("⚙  1000 RPM DC motors · 2S Li-Po", "⚙  1000 RPM DC motorlar · 2S Li-Po"),
            bi("🧱 3D-printed chassis · under 500 g", "🧱 3D baskı şasi · 500 g altı"),
            bi("🧮 Sensor fusion → threat score → attack/search", "🧮 Sensör füzyonu → tehdit skoru → saldırı/arama"),
        ],
    },
    Competition {
        number: "ROBOTİK / 03",
        name: "MEB Robot",
        year: "2026",
        kind: bi("Robotics", "Robotik"),
        glyph: "✺",
        tone: (0x8D, 0x50, 0x39),
        id_code: "ALT-2026-03",
        blurb: bi(
            "The next lap of the robotics journey.",
            "Robotik yolculuğunun bir sonraki turu.",
        ),
        award: None,
        back: &[
            bi("🤖 Robotics track · 2026 season", "🤖 Robotik kategorisi · 2026 sezonu"),
            bi("🧱 Built from scratch, hardware + firmware", "🧱 Sıfırdan; donanım + yazılım"),
            bi("📝 Details land here after the event", "📝 Ayrıntılar etkinlikten sonra burada"),
        ],
    },
];

// ───────────────────────────────── Dev Logs ─────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct DevLog {
    pub title: Bi,
    pub category: Bi,
    pub date: &'static str,
    pub read_min: u8,
    pub tags: &'static [&'static str],
    pub excerpt: Bi,
    pub sections: &'static [(Bi, Bi)],
    pub callout: Bi,
}

pub static DEV_LOGS: [DevLog; 5] = [
    DevLog {
        title: bi(
            "Building a 36-hour AI Agent Prototype",
            "36 Saatte Çalışan AI Ajanı Prototipi Geliştirmek",
        ),
        category: bi("Devpost & Hackathons", "Devpost & Hackathons"),
        date: "2026-07-22",
        read_min: 8,
        tags: &["Hackathon", "AI Agent", "Devpost", "Sprint"],
        excerpt: bi(
            "Shipping an AI agent prototype from scratch in a hackathon: planning, fast iteration, sleep management and last-minute debugging stories.",
            "Hackathon ortamında sıfırdan bir AI agent prototipi çıkarmak: planlama, hızlı iterasyon, uyku yönetimi ve son dakika debug hikâyeleri.",
        ),
        sections: &[
            (
                bi("Planning: the first 6 hours", "Proje Planlaması: İlk 6 Saat"),
                bi(
                    "We locked the architecture first: which LLM API, the agent's decision tree, and a deliberately minimal UI. Stack: Python + Flask backend, Gemini API (free tier fits a hackathon), vanilla JS + minimal CSS frontend with zero framework overhead.",
                    "İlk iş mimari kararları aldık: hangi LLM API, ajanın karar ağacı ve bilinçli olarak minimal bir arayüz. Teknoloji: Python + Flask, Gemini API (ücretsiz tier hackathon'a uygun), vanilla JS + minimal CSS.",
                ),
            ),
            (
                bi("Build: 20 hours non-stop", "Geliştirme: 20 Saat Non-Stop"),
                bi(
                    "We implemented the ReAct (Reasoning and Acting) pattern: the agent first thinks, then plans an action, executes it, and loops with the result in its history until the task is complete.",
                    "ReAct (Reasoning and Acting) pattern'ini uyguladık: ajan önce düşünür, sonra aksiyonu planlar, çalıştırır ve görev bitene kadar sonucu geçmişine ekleyerek döngüye devam eder.",
                ),
            ),
            (
                bi("Demo & lessons", "Demo ve Çıkarılan Dersler"),
                bi(
                    "In front of the jury the agent ran a live web research, summarized it and reported back. The lesson: ship the MVP fast, then iterate — a working demo beats perfectionism.",
                    "Jüri önünde ajan canlı bir web araştırması yaptı, özetledi ve raporladı. Ders: MVP'yi hızlı çıkar, sonra iterasyon yap — çalışan bir demo mükemmeliyetçilikten değerlidir.",
                ),
            ),
        ],
        callout: bi(
            "\"6 hours of planning, 20 hours of code, 10 hours of debugging and demo prep.\"",
            "\"İlk 6 saatte plan yaptık, sonraki 20 saatte kod yazdık, son 10 saatte debug ettik ve demo hazırladık.\"",
        ),
    },
    DevLog {
        title: bi(
            "NexusAI-PhoneAgent: Autonomous LLM Agents on Mobile",
            "NexusAI-PhoneAgent: Mobil Cihazlarda Otonom LLM Ajanları",
        ),
        category: bi("AI & Agents", "AI & Agents"),
        date: "2026-06-15",
        read_min: 10,
        tags: &["React Native", "LLM", "Mobile AI", "Expo"],
        excerpt: bi(
            "Building an autonomous mobile AI agent from scratch with React Native: architecture, LLM integration and on-device OS interaction.",
            "React Native ile sıfırdan otonom mobil AI ajanı geliştirmek: mimari kararlar, LLM entegrasyonu ve cihaz üzerinde işletim sistemi etkileşimi.",
        ),
        sections: &[
            (
                bi("Architecture", "Mimari Tasarım"),
                bi(
                    "React Native + Expo app with an agent layer that talks to an LLM service over JSON-RPC. Natural-language commands go through intent parsing (tokenize → classify → map to action → execute natively).",
                    "React Native + Expo ile geliştirildi; arka planda JSON-RPC ile bir LLM servisiyle konuşan agent katmanı var. Doğal dil komutları intent parsing'den geçip (tokenize → sınıflandır → aksiyona eşle → cihazda çalıştır) native modüllere çevriliyor.",
                ),
            ),
            (
                bi("LLM integration", "LLM Entegrasyonu"),
                bi(
                    "Cloud LLM APIs instead of on-device models, for quality and battery. A summarization layer optimizes the context window before every call.",
                    "Model kalitesi ve batarya için cihazda model yerine bulut LLM API'leri kullanıldı. Her çağrıdan önce context window'u optimize eden bir özetleme katmanı var.",
                ),
            ),
            (
                bi("Challenges & next steps", "Zorluklar ve Gelecek Planlar"),
                bi(
                    "Latency solved with streaming responses; long tasks run through Expo background tasks. Next: on-device LLM inference for fully offline agent commands.",
                    "Gecikme streaming response ile çözüldü; uzun işler Expo background task API'leriyle yönetiliyor. Sıradaki adım: tam offline çalışma için on-device LLM inference.",
                ),
            ),
        ],
        callout: bi(
            "\"A phone is not just a screen — with its sensors, camera and connectivity it is a full autonomous agent platform.\"",
            "\"Telefon sadece bir ekran değil — sensörleri, kamerası ve bağlantılarıyla tam bir otonom agent platformu.\"",
        ),
    },
    DevLog {
        title: bi(
            "Fibonacci Robot Cup: Sensor Fusion and a Fast Decision Engine",
            "Fibonacci Robot Kupası: Sensör Füzyonu ve Hızlı Karar Motoru",
        ),
        category: bi("Robotics", "Robotics"),
        date: "2026-05-08",
        read_min: 7,
        tags: &["Arduino", "C++", "Sensor Fusion", "Robotics"],
        excerpt: bi(
            "Winning the jury special award with an autonomous mini sumo robot: sensor fusion, motor control algorithms and tournament strategy.",
            "Otonom mini sumo robotunda jüri özel ödülü kazanmak: sensör füzyonu, motor kontrol algoritmaları ve turnuva stratejileri.",
        ),
        sections: &[
            (
                bi("Hardware design", "Donanım Tasarımı"),
                bi(
                    "3D-printed chassis, 1000 RPM DC motors and a 2S Li-Po. Every component was optimized gram by gram to stay under the 500 g limit. Sensors: 4× QTR line sensors, 3× Sharp distance sensors (front, left, right) and an MPU6050 IMU.",
                    "Şasi 3D printer ile üretildi; 1000 RPM DC motorlar ve 2S Li-Po kullanıldı. 500 g sınırı için her bileşen gram gram optimize edildi. Sensörler: 4× QTR çizgi, 3× Sharp mesafe (ön, sol, sağ) ve MPU6050 IMU.",
                ),
            ),
            (
                bi("Sensor fusion algorithm", "Sensör Füzyonu Algoritması"),
                bi(
                    "The brain reads and fuses every sensor hundreds of times per second. Edge detected → emergency reverse. Otherwise a weighted threat score (1/front·0.6 + 1/left·0.2 + 1/right·0.2) with IMU heading decides: full-speed attack or search pattern.",
                    "Robotun beyni tüm sensörleri saniyede yüzlerce kez okuyup birleştiriyor. Çizgi algılanırsa acil geri manevra; aksi halde IMU yönüyle birlikte ağırlıklı tehdit skoru (1/ön·0.6 + 1/sol·0.2 + 1/sağ·0.2) saldırı ya da arama kararını veriyor.",
                ),
            ),
            (
                bi("Tournament day", "Turnuva Günü"),
                bi(
                    "5 of 7 matches won. The jury special award came from the from-scratch code and the uniqueness of the sensor fusion approach, not just performance.",
                    "7 maçın 5'ini kazandık. Jüri özel ödülü sadece performanstan değil, robotun tamamen sıfırdan kodlanmasından ve sensör füzyonu yaklaşımının eşsizliğinden geldi.",
                ),
            ),
        ],
        callout: bi(
            "\"The winning robot isn't the strongest — it's the fastest to decide.\"",
            "\"Kazanan robot, en güçlü olan değil — en hızlı karar veren.\"",
        ),
    },
    DevLog {
        title: bi(
            "A Linux Dev Environment from Scratch: Hyprland & Keyboard-First Productivity",
            "Sıfırdan Linux Geliştirici Ortamı: Hyprland ve Klavye Odaklı Üretkenlik",
        ),
        category: bi("Linux & Setup", "Linux & Setup"),
        date: "2026-04-20",
        read_min: 6,
        tags: &["Linux", "Hyprland", "Wayland", "Dotfiles"],
        excerpt: bi(
            "A custom Arch Linux + Hyprland WM setup: dotfile management, terminal workflow and developer tooling configuration.",
            "Custom Arch Linux + Hyprland WM kurulumu: dotfile yönetimi, terminal workflow'u ve geliştirici araçları konfigürasyonu.",
        ),
        sections: &[
            (
                bi("Why a tiling WM?", "Neden Tiling Window Manager?"),
                bi(
                    "GNOME and KDE are nice but heavy. My screen only holds a terminal, an editor and a browser — I don't need a full desktop environment to manage them.",
                    "GNOME ve KDE güzel ama ağır. Ekranımda sadece terminal, editör ve tarayıcı var — bunları yönetmek için tam teşekküllü bir masaüstü ortamına ihtiyacım yok.",
                ),
            ),
            (
                bi("Hyprland & terminal workflow", "Hyprland ve Terminal Workflow"),
                bi(
                    "Arch Linux with a hand-configured Hyprland (Wayland, animated, GPU-accelerated). Borders use the brand colors DA291C → 5E7060. Kitty + Zsh + Starship; one workspace per project, one window per task — no tmux.",
                    "Arch Linux üzerinde elle yapılandırılmış Hyprland (Wayland, animasyonlu, GPU hızlandırmalı). Kenarlıklar marka renkleri DA291C → 5E7060. Kitty + Zsh + Starship; her workspace bir proje, her pencere bir görev — tmux yok.",
                ),
            ),
            (
                bi("Dotfiles", "Dotfile Yönetimi"),
                bi(
                    "Every config lives in a Git repo; a single `stow` command restores a whole machine. Reproducible development environments.",
                    "Tüm konfigürasyonlar bir Git repo'sunda; tek bir `stow` komutuyla yeni makine kurulur. Reproducible development environment'ın gücü.",
                ),
            ),
        ],
        callout: bi(
            "\"The less you use the mouse, the faster you build.\"",
            "\"Fareyi ne kadar az kullanırsanız, o kadar hızlı geliştirirsiniz.\"",
        ),
    },
    DevLog {
        title: bi(
            "The 'From-Scratch' Philosophy in Modern Web — Without Heavy Libraries",
            "Modern Web'de 'From-Scratch' Felsefesi: Ağır Kütüphaneler Olmadan",
        ),
        category: bi("Web & Architecture", "Web & Architecture"),
        date: "2026-03-10",
        read_min: 9,
        tags: &["JavaScript", "CSS3", "Performance", "From-Scratch"],
        excerpt: bi(
            "Building 3D effects and fluid interfaces without a framework stack: CSS transforms, vanilla JS event handling and performance optimization.",
            "3D efektler ve akıcı arayüzleri framework yığını olmadan inşa etmek: CSS transforms, vanilla JS event handling ve performans optimizasyonu.",
        ),
        sections: &[
            (
                bi("Why go framework-less", "Framework'süz Geliştirmenin Avantajları"),
                bi(
                    "React, Vue, Svelte are great — but shipping 200KB+ of JavaScript for a portfolio or landing page rarely makes sense.",
                    "React, Vue, Svelte harika araçlar; ama bir portföy ya da landing page için 200KB+ JavaScript göndermek çoğu zaman mantıksız.",
                ),
            ),
            (
                bi("3D card deck in pure math", "Saf Matematikle 3D Kart Sistemi"),
                bi(
                    "The portfolio's 3D card deck is plain CSS transforms plus wheel-event handling: each card shifts back (translateZ), up (translateY), scales down and fades by its offset from the active card. The whole site is ~70KB of JS, Lighthouse 98/100.",
                    "Portföydeki 3D kart destesi saf CSS transforms ve wheel event ile yapıldı: her kart aktif karta uzaklığına göre geriye (translateZ), yukarı (translateY) kayıyor, küçülüyor ve soluklaşıyor. Sitenin toplam JS'i ~70KB, Lighthouse 98/100.",
                ),
            ),
            (
                bi("When to use a framework", "Ne Zaman Framework Kullanmalı?"),
                bi(
                    "Complex state, many forms and data flows, or a big team — use a framework. For a showcase, blog or landing page, vanilla wins.",
                    "State yönetimi karmaşıksa, çok sayıda form ve veri akışı varsa ya da büyük bir ekiple çalışılıyorsa framework kullan. Showcase, blog veya landing page için vanilla kazanır.",
                ),
            ),
        ],
        callout: bi(
            "\"The fastest code is the code you never ship. The best dependency is the one you never add.\"",
            "\"En hızlı kod, hiç gönderilmeyen koddur. En iyi dependency, hiç eklenmeyendir.\"",
        ),
    },
];

// ───────────────────────────────── Contact ─────────────────────────────────

#[derive(Debug, Clone, Copy)]
pub struct Contact {
    pub icon: &'static str,
    pub label: Bi,
    pub value: &'static str,
    pub url: &'static str,
    pub note: Bi,
}

pub static CONTACTS: [Contact; 7] = [
    Contact {
        icon: "🌐",
        label: bi("Website", "Web Sitesi"),
        value: "alttre.is-a.dev",
        url: "https://alttre.is-a.dev/",
        note: bi(
            "The full portfolio — also in Turkish at /tr/ and /about/.",
            "Tam portföy — Türkçesi /tr/ ve /about/ adreslerinde.",
        ),
    },
    Contact {
        icon: "✉",
        label: bi("Email", "E-posta"),
        value: "ali-turan-@outlook.com",
        url: "mailto:ali-turan-@outlook.com",
        note: bi(
            "Best way to reach me for internships and projects.",
            "Staj ve projeler için bana ulaşmanın en iyi yolu.",
        ),
    },
    Contact {
        icon: "🐙",
        label: bi("GitHub", "GitHub"),
        value: "github.com/Alttrest",
        url: "https://github.com/Alttrest",
        note: bi("All source code lives here (@Alttrest).", "Tüm kaynak kodlar burada (@Alttrest)."),
    },
    Contact {
        icon: "🏁",
        label: bi("Devpost", "Devpost"),
        value: "devpost.com/Alttrest",
        url: "https://devpost.com/Alttrest",
        note: bi("Hackathon projects and write-ups.", "Hackathon projeleri ve yazıları."),
    },
    Contact {
        icon: "💼",
        label: bi("LinkedIn", "LinkedIn"),
        value: "Ali Turan",
        url: "https://linkedin.com",
        note: bi("Professional profile — search \"Ali Turan\".", "Profesyonel profil — \"Ali Turan\" diye ara."),
    },
    Contact {
        icon: "📸",
        label: bi("Instagram", "Instagram"),
        value: "@Alttre",
        url: "https://instagram.com",
        note: bi("Builds, hardware and behind the scenes.", "Yapımlar, donanım ve kamera arkası."),
    },
    Contact {
        icon: "🎮",
        label: bi("Discord", "Discord"),
        value: "alttre.is-a.dev (verified domain)",
        url: "https://alttre.is-a.dev/.well-known/discord",
        note: bi(
            "Domain verified via .well-known/discord — no public handle.",
            "Alan adı .well-known/discord ile doğrulanmış — herkese açık kullanıcı adı yok.",
        ),
    },
];

pub const CONTACT_PROMPT: Bi = bi(
    "Hello, my name is [name]. You can reach me at [email]. I would like to discuss [message].",
    "Merhaba, adım [ad]. Bana [e-posta] adresinden ulaşabilirsiniz. [mesaj] hakkında konuşmak istiyorum.",
);

pub const CONTACT_NOTE: Bi = bi(
    "Based in Türkiye — open to software developer internships and opportunities in Istanbul.",
    "Türkiye'deyim — İstanbul'da yazılım geliştirici stajı ve fırsatlarına açığım.",
);

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_counts_match_website() {
        assert_eq!(PROJECTS.len(), 13);
        assert_eq!(SKILL_CATEGORIES.len(), 5);
        assert_eq!(skill_count(), 22);
        assert_eq!(COMPETITIONS.len(), 3);
        assert_eq!(DEV_LOGS.len(), 5);
        assert_eq!(INTRO_LINES.len(), 5);
        assert_eq!(CONTACTS.len(), 7);
    }

    #[test]
    fn filters_partition_projects() {
        let all = filtered_projects(None).len();
        let sum: usize = PROJECT_FILTERS[1..]
            .iter()
            .map(|f| filtered_projects(*f).len())
            .sum();
        assert_eq!(all, sum);
    }

    #[test]
    fn skill_links_reference_real_skills() {
        let names: Vec<&str> = SKILL_CATEGORIES
            .iter()
            .flat_map(|c| c.skills.iter().map(|s| s.name))
            .collect();
        for (a, b) in SKILL_LINKS {
            assert!(names.contains(&a), "{a}");
            assert!(names.contains(&b), "{b}");
        }
        assert_eq!(linked_skills("Python"), vec!["Flask", "ReAct Agents"]);
    }

    #[test]
    fn lang_toggle_roundtrips() {
        assert_eq!(Lang::En.toggle().toggle(), Lang::En);
        assert_eq!(INTRO_LINES[0].get(Lang::Tr), "Merhaba, ben Ali Turan (Alttre).");
    }
}
