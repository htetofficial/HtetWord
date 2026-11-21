//! Built-in sample documents and templates (original text, generated in code).

use std::sync::Arc;
use wordcraft_doc::para::InlineObject;
use wordcraft_doc::props::{Align, Border, BorderStyle, Borders, CharProps, NumRef, Rgb, TabAlign, TabLeader, TabStop, TextColor};
use wordcraft_doc::{Block, Document, ListKind, Paragraph, PartKind, Table, para_block};

fn para(text: &str) -> Paragraph {
    Paragraph::with_text(text, CharProps::default())
}

fn styled(text: &str, style: &str) -> Paragraph {
    para(text).styled(style)
}

/// Append text with formatting to a paragraph.
fn add(p: &mut Paragraph, text: &str, props: CharProps) {
    let at = p.len();
    let _ = p.insert_text(at, text, &props);
}

fn bold() -> CharProps {
    CharProps { bold: Some(true), ..Default::default() }
}
fn italic() -> CharProps {
    CharProps { italic: Some(true), ..Default::default() }
}

/// The showcase document: a short studio handbook with headings, lists, a table, a quote,
/// a footer with page numbers.
pub fn sample_document() -> Document {
    let mut d = Document::new();
    d.core.title = "The Open Studio Handbook".into();
    d.core.creator = "WordCraft".into();
    let mut b: Vec<Block> = Vec::new();
    b.push(Block::Para(styled("The Open Studio Handbook", "Title")));
    b.push(Block::Para(styled("How a small community of makers shares space, tools and time", "Subtitle")));
    let mut intro = para("Every Thursday evening the old print shop on Alder Street fills with people who make things. ");
    add(&mut intro, "Painters", bold());
    add(&mut intro, " set up by the north windows, ", CharProps::default());
    add(&mut intro, "letterpress", italic());
    add(
        &mut intro,
        " regulars claim the back room, and somebody always brings bread. This handbook collects what we have learned about running a shared studio: the routines that keep it pleasant, the tools we pool, and the small agreements that let thirty strangers become a crew.",
        CharProps::default(),
    );
    b.push(Block::Para(intro));
    b.push(Block::Para(styled("Why share a studio?", "Heading1")));
    b.push(Block::Para(para("Rent is the obvious reason, but it is not the best one. A shared room puts your work next to work you would never have seen otherwise. A ceramicist explains glaze chemistry to a comic artist; a photographer borrows a weaver's loom for a still life. The cross-talk is the point.")));
    let mut quote = styled("A studio is a place where unfinished things are welcome.", "Quote");
    quote.props.space_before = Some(12.0);
    b.push(Block::Para(quote));
    b.push(Block::Para(styled("What we agree on", "Heading2")));
    let mut num = (*d.numbering).clone();
    let bullets = num.add_list(ListKind::Bullet);
    let steps = num.add_list(ListKind::Numbered);
    d.numbering = Arc::new(num);
    for (lead, rest) in [
        ("Clean as you go. ", "Leave a table the way you would like to find it."),
        ("Label what is yours. ", "Unlabelled supplies on the commons shelf are fair game."),
        ("Ask before you borrow. ", "Even from the shelf, a quick note helps everyone keep track."),
        ("Quiet hours ", "run from 9 to 11 on weekday mornings."),
    ] {
        let mut p = Paragraph::new().styled("ListParagraph");
        add(&mut p, lead, bold());
        add(&mut p, rest, CharProps::default());
        p.props.numbering = Some(NumRef { num: bullets, level: 0 });
        b.push(Block::Para(p));
    }
    b.push(Block::Para(styled("Shared equipment", "Heading2")));
    b.push(Block::Para(para("The table below lists what the membership owns together and who keeps each item in working order.")));
    let rows = [
        ["Equipment", "Location", "Steward", "Booking"],
        ["Etching press", "Back room", "Mara O.", "Sign-up sheet"],
        ["Kiln (cone 10)", "Yard shed", "Theo P.", "48 h notice"],
        ["Large-format printer", "Office", "Ines R.", "Calendar"],
        ["Light table", "North windows", "Anyone", "First come"],
    ];
    let mut t = Table::new(rows.len(), 4, 468.0);
    t.props.style = Some("GridTable4AccentBlue".into());
    for (r, row) in rows.iter().enumerate() {
        for (c, txt) in row.iter().enumerate() {
            if let Some(cell) = t.rows.get_mut(r).and_then(|x| x.cells.get_mut(c)) {
                cell.blocks = vec![para_block(para(txt))];
            }
        }
    }
    if let Some(r) = t.rows.first_mut() {
        r.props.header = true;
    }
    b.push(Block::Table(t));
    b.push(Block::Para(styled("Opening the studio", "Heading2")));
    for s in [
        "Switch on the main breaker by the door.",
        "Open both skylights unless it is raining.",
        "Start the kettle. This step is not optional.",
        "Check the booking sheet for the kiln and press.",
    ] {
        let mut p = para(s).styled("ListParagraph");
        p.props.numbering = Some(NumRef { num: steps, level: 0 });
        b.push(Block::Para(p));
    }
    b.push(Block::Para(styled("Membership", "Heading1")));
    let mut m = para("Membership is ");
    add(&mut m, "pay what you can", CharProps { bold: Some(true), color: Some(TextColor::Rgb(Rgb(0x15, 0x60, 0x82))), ..Default::default() });
    add(
        &mut m,
        ", reviewed every season. New members shadow a regular for their first two visits, and everyone takes one cleaning shift a month. If you have never made anything before, you are exactly who we hoped would show up.",
        CharProps::default(),
    );
    b.push(Block::Para(m));
    let mut h = para("Questions go to the front desk, or to the notice board by the sink. ");
    add(
        &mut h,
        "Visit the community board",
        CharProps { style: Some("Hyperlink".into()), link: Some("https://getartcraft.com/".into()), ..Default::default() },
    );
    add(&mut h, " for events.", CharProps::default());
    b.push(Block::Para(h));
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    // Footer: centred page number.
    let mut f = Paragraph::new().styled("Footer");
    f.props.align = Some(Align::Center);
    let _ = f.insert_object(
        0,
        InlineObject::Field { instr: "PAGE".into(), result: "1".into(), locked: false, dirty: false, code: false, form: String::new() },
        &CharProps::default(),
    );
    let fid = d.add_part(PartKind::Footer, vec![para_block(f)]);
    d.last_section.footers.default = Some(fid);
    let mut hd = styled("The Open Studio Handbook\t\tSpring edition", "Header");
    hd.props.borders = Some(Borders {
        bottom: Some(Border { style: BorderStyle::Single, width: 0.5, color: Some(Rgb(0xBF, 0xBF, 0xBF)), space: 4.0 }),
        ..Default::default()
    });
    let hid = d.add_part(PartKind::Header, vec![para_block(hd)]);
    d.last_section.headers.default = Some(hid);
    d
}

/// A block-style letter.
pub fn letter() -> Document {
    let mut d = Document::new();
    let lines: Vec<Paragraph> = vec![
        styled("Your Name", "Title"),
        para("123 Your Street · City, ST 00000 · you@example.com"),
        para(""),
        para(&crate::cmd::insert::format_date("MMMM d, yyyy")),
        para(""),
        para("Recipient Name"),
        para("Title, Company"),
        para("Street Address"),
        para(""),
        para("Dear Recipient,"),
        para(
            "Start with a sentence that says why you are writing. Keep paragraphs short and specific, and close with what you would like to happen next.",
        ),
        para("Thank you for your time."),
        para("Sincerely,"),
        para(""),
        para("Your Name"),
    ];
    d.body = lines.into_iter().map(para_block).collect();
    d
}

/// A one-page résumé.
pub fn resume() -> Document {
    let mut d = Document::new();
    let mut b: Vec<Block> =
        vec![Block::Para(styled("Alex Rivera", "Title")), Block::Para(para("Illustrator · alex@example.com · portfolio.example.com"))];
    for (head, items) in [
        ("Experience", vec![("Lead Illustrator, Harbor Books", "2021 – present"), ("Freelance Illustrator", "2016 – 2021")]),
        ("Education", vec![("BFA Illustration, Coastal College of Art", "2016")]),
        ("Skills", vec![("Ink, gouache, digital painting, book layout, lettering", "")]),
    ] {
        b.push(Block::Para(styled(head, "Heading1")));
        for (what, when) in items {
            let mut p = para(what);
            p.props.tabs = Some(vec![TabStop { pos: 468.0, align: TabAlign::Right, leader: TabLeader::None }]);
            if !when.is_empty() {
                add(&mut p, &format!("\t{when}"), italic());
            }
            b.push(Block::Para(p));
        }
    }
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

/// A report with a cover heading and sections.
pub fn report() -> Document {
    let mut d = Document::new();
    let b: Vec<Block> = vec![
        Block::Para(styled("Report Title", "Title")),
        Block::Para(styled("Subtitle or date", "Subtitle")),
        Block::Para(styled("Summary", "Heading1")),
        Block::Para(para("Summarise the findings in two or three sentences.")),
        Block::Para(styled("Background", "Heading1")),
        Block::Para(para("Describe the context and the question the report answers.")),
        Block::Para(styled("Findings", "Heading1")),
        Block::Para(styled("First finding", "Heading2")),
        Block::Para(para("Explain the evidence.")),
        Block::Para(styled("Recommendations", "Heading1")),
        Block::Para(para("List what should happen next.")),
    ];
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

/// Myanmar Article template (ဆောင်းပါး ပုံစံခွက်).
pub fn myanmar_article() -> Document {
    let mut d = Document::new();
    d.core.title = "မြန်မာ့ယဉ်ကျေးမှုနှင့် စာပေအနုပညာ".into();
    d.core.creator = "HtetWord".into();
    let b: Vec<Block> = vec![
        Block::Para(styled("မြန်မာ့ယဉ်ကျေးမှုနှင့် စာပေအနုပညာ", "Title")),
        Block::Para(styled("ခေတ်အဆက်ဆက် ထိန်းသိမ်းလက်ဆင့်ကမ်းလာသော တန်ဖိုးမဖြတ်နိုင်သည့် စာပေအမွေအနှစ်များ", "Subtitle")),
        Block::Para(para("ရေးသားသူ - မမိုးသူဇာ · ဖေဖော်ဝါရီ ၂၀၂၆ · HtetWord စာပေမဂ္ဂဇင်း")),
        Block::Para(styled("နိဒါန်း", "Heading1")),
        Block::Para(para("မြန်မာနိုင်ငံ၏ စာပေသမိုင်းကြောင်းသည် ရှေးပုဂံခေတ် ကျောက်စာများမှသည် ယနေ့ခေတ် ဒစ်ဂျစ်တယ်စာပေခေတ်တိုင်အောင် ကြွယ်ဝပြည့်စုံစွာ ရှင်သန်ဖွံ့ဖြိုးခဲ့ပါသည်။ စာပေသည် လူမျိုးတစ်မျိုး၏ ယဉ်ကျေးမှုအဆင့်အတန်း၊ တွေးခေါ်မြော်မြင်မှုနှင့် နှလုံးသားခံစားချက်တို့ကို ထင်ဟပ်ဖော်ပြသည့် အကြည်လင်ဆုံး မှန်ချပ်ကြီးတစ်ခုပင် ဖြစ်သည်။")),
        Block::Para(styled("“စာပေမြင့်မှ လူမျိုးတင့်မည် ဟူသော စကားရပ်သည် မြန်မာတို့၏ စာပေတန်ဖိုးထားမှုကို ပေါ်လွင်စေသည်။”", "Quote")),
        Block::Para(styled("စာပေ၏ အခန်းကဏ္ဍနှင့် အရေးပါမှု", "Heading1")),
        Block::Para(para("ခေတ်မီဖွံ့ဖြိုးသော လူ့ဘောင်အဖွဲ့အစည်းတစ်ခု တည်ဆောက်ရာတွင် စာဖတ်ခြင်းနှင့် စာပေဗဟုသုတ လေ့လာဆည်းပူးခြင်းသည် အဓိကမောင်းနှင်အား ဖြစ်သည်။")),
        Block::Para(styled("၁။ အသိပညာဗဟုသုတ တိုးပွားစေခြင်း", "Heading2")),
        Block::Para(para("စာပေလေ့လာဖတ်ရှုခြင်းဖြင့် မိမိမရောက်ဖူးသော နေရာဒေသများ၊ မသိသေးသော ဗဟုသုတများနှင့် ကမ္ဘာ့အမြင်များကို အလွယ်တကူ ရရှိခံစားနိုင်သည်။")),
        Block::Para(styled("၂။ စိတ်နှလုံး နူးညံ့သိမ်မွေ့စေခြင်း", "Heading2")),
        Block::Para(para("ရသစာပေများသည် လူသားတို့၏ စိတ်နှလုံးကို နူးညံ့စေပြီး ကိုယ်ချင်းစာတရားနှင့် မေတ္တာတရားတို့ကို တိုးပွားစေပါသည်။")),
        Block::Para(styled("နိဂုံး", "Heading1")),
        Block::Para(para("သို့ဖြစ်ပါ၍ မိမိတို့၏ မြန်မာစာပေ၊ မြန်မာစကားကို ချစ်မြတ်နိုးစွာ ထိန်းသိမ်းကြပြီး ခေတ်နှင့်အညီ တိုးတက်ပြောင်းလဲနေသော နည်းပညာများနှင့် ပေါင်းစပ်ကာ မြန်မာ့စာပေလောကကို ဆက်လက်မြှင့်တင်ကြရမည် ဖြစ်ပါသည်။")),
    ];
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

/// Myanmar Book / Novel Chapter template (စာအုပ် / ဝတ္ထု ပုံစံခွက်).
pub fn myanmar_book() -> Document {
    let mut d = Document::new();
    d.core.title = "မေတ္တာရိပ်နွေး ရာသီသစ်".into();
    d.core.creator = "HtetWord".into();
    let b: Vec<Block> = vec![
        Block::Para(styled("မေတ္တာရိပ်နွေး ရာသီသစ်", "Title")),
        Block::Para(styled("ဝတ္ထုရှည် (ပထမတွဲ)", "Subtitle")),
        Block::Para(para("စာရေးသူ - ထက်မြတ်သူ")),
        Block::Para(styled("မာတိကာ", "Heading1")),
        Block::Para(para("အခန်း (၁) - အရုဏ်ဦး၏ နှုတ်ဆက်သံ ............................ စာမျက်နှာ ၁")),
        Block::Para(para("အခန်း (၂) - မမျှော်လင့်သော တွေ့ဆုံခြင်း ..................... စာမျက်နှာ ၅")),
        Block::Para(para("အခန်း (၃) - ရာသီသစ်၏ မျှော်လင့်ချက် ........................ စာမျက်နှာ ၁၂")),
        Block::Para(styled("အခန်း (၁) - အရုဏ်ဦး၏ နှုတ်ဆက်သံ", "Heading1")),
        Block::Para(para("ဆောင်းနှင်းမြူတို့ သင်းပျံ့နေသော မနက်ခင်းတစ်ခုတွင် ရွာကလေး၏ လမ်းမထက်၌ ငှက်ကလေးများ၏ တေးသံသည် သာယာစွာ ပျံ့လွင့်နေသည်။ သစ်ရွက်စိမ်းတို့အပေါ်တွင် တွဲခိုနေသော နှင်းစက်ကလေးများသည် နံနက်ခင်းနေရောင်ခြည်အောက်တွင် စိန်ပွင့်လေးများပမာ တလက်လက် တောက်ပနေကြသည်။")),
        Block::Para(para("“မိုးသောက်၊ မနက်စာ စားဖို့ အချိန်တန်ပြီနော်” ဟု အမေဖြစ်သူ၏ နွေးထွေးသော အသံကို ကြားလိုက်ရချိန်တွင် မိုးသောက်တစ်ယောက် စာကြည့်စားပွဲရှေ့မှ မတ်တတ်ထရပ်လိုက်မိသည်။")),
        Block::Para(para("“ဟုတ်ကဲ့ပါ အမေ၊ သား အခုပဲ လာခဲ့ပါမယ်” ဟု သူက ခပ်တိုးတိုး ပြန်လည်ဖြေကြားရင်း စားပွဲပေါ်ရှိ ဒိုင်ယာရီစာအုပ်အပြာရောင်လေးကို အသာအယာ ပိတ်လိုက်လေသည်။")),
        Block::Para(para("ဤစာအုပ်ကလေးထဲတွင် သူ၏ ရည်မှန်းချက်များ၊ အိပ်မက်များနှင့် ဘဝ၏ အဖိုးတန်သော အမှတ်တရများစွာ ကိန်းအောင်းနေသည်ကို မည်သူမျှ မသိနိုင်ပေ။")),
    ];
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

/// Myanmar Social Media & Blog Post template (ဆိုရှယ်ပို့စ် / ဘလော့ဂ် ပုံစံခွက်).
pub fn myanmar_post() -> Document {
    let mut d = Document::new();
    d.core.title = "နေ့စဉ်ဘဝမှာ အောင်မြင်ပျော်ရွှင်စေမယ့် အလေ့အကျင့် ၅ ခု 🌸".into();
    d.core.creator = "HtetWord".into();
    let b: Vec<Block> = vec![
        Block::Para(styled("နေ့စဉ်ဘဝမှာ အောင်မြင်ပျော်ရွှင်စေမယ့် အလေ့အကျင့် ၅ ခု 🌸", "Title")),
        Block::Para(styled("အချိန်ကို အကျိုးရှိရှိ အသုံးချပြီး စိတ်ချမ်းသာမှုရရှိစေမယ့် လျှို့ဝှက်ချက်များ", "Subtitle")),
        Block::Para(para("လူတိုင်းမှာ တစ်နေ့ကို ၂၄ နာရီစီ အညီအမျှ ရရှိထားကြပါတယ်။ ဒါပေမယ့် ဘာကြောင့် တချို့လူတွေက ပိုပြီး အောင်မြင်ပျော်ရွှင်နေကြတာလဲ? အဓိကကတော့ နေ့စဉ် ပြုလုပ်တဲ့ အလေ့အကျင့်လေးတွေကြောင့်ပါပဲ။")),
        Block::Para(styled("✨ ၁။ မနက်စောစော ထခြင်း", "Heading2")),
        Block::Para(para("မနက်ခင်း အစောပိုင်း အချိန်လေးဟာ စိတ်ကြည်လင်အေးချမ်းပြီး တစ်နေ့တာအတွက် အစီအစဉ်ဆွဲဖို့ အကောင်းဆုံး အချိန်ဖြစ်ပါတယ်။")),
        Block::Para(styled("✨ ၂။ စာဖတ်ခြင်း (အနည်းဆုံး ၁၅ မိနစ်)", "Heading2")),
        Block::Para(para("စာအုပ်ကောင်းတစ်အုပ်ကနေ ရရှိတဲ့ အတွေးအခေါ်တစ်ခုဟာ သင့်ဘဝလမ်းကြောင်းကို ပြောင်းလဲပေးနိုင်ပါတယ်။")),
        Block::Para(styled("✨ ၃။ ကျေးဇူးတင်တတ်သော စိတ်ထား မွေးမြူခြင်း", "Heading2")),
        Block::Para(para("မိမိပိုင်ဆိုင်ထားတဲ့ အရာသေးသေးလေးတွေအတွက် နေ့စဉ် ကျေးဇူးတင်မှတ်တမ်းရေးကြည့်ပါ။ စိတ်ဖိစီးမှုတွေ သိသိသာသာ လျော့ကျသွားပါလိမ့်မယ်။")),
        Block::Para(styled("✨ ၄။ ကျန်းမာရေးအတွက် လှုပ်ရှားလေ့ကျင့်ခြင်း", "Heading2")),
        Block::Para(para("လမ်းလျှောက်ခြင်း သို့မဟုတ် ယောဂကျင့်ခြင်းကဲ့သို့ လေ့ကျင့်ခန်း ပုံမှန်လုပ်ပေးခြင်းဖြင့် စိတ်ရောကိုယ်ပါ လန်းဆန်းတက်ကြွစေပါတယ်။")),
        Block::Para(styled("✨ ၅။ ညအိပ်ရာမဝင်မီ ဖုန်းအသုံးပြုမှု လျှော့ချခြင်း", "Heading2")),
        Block::Para(para("အိပ်ရာမဝင်မီ မိနစ် ၃၀ ခန့် screen time လျှော့ချလိုက်ရင် ညဘက် နှစ်ခြိုက်စွာ အိပ်ပျော်စေပါတယ်။")),
        Block::Para(styled("💬 သင့်အတွက် မေးခွန်းလေး", "Heading1")),
        Block::Para(para("ဒီအလေ့အကျင့် ၅ ခုထဲက သင် နေ့စဉ် လုပ်ဖြစ်နေတဲ့ အလေ့အကျင့်လေးက ဘာများဖြစ်မလဲ? comment မှာ မျှဝေပေးခဲ့ပါဦးနော်! 💕")),
        Block::Para(para("#HtetWord #စာပေ #ဘဝနေနည်း #အတွေးအမြင် #အောင်မြင်ရေး #မြန်မာစာ")),
    ];
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

/// Myanmar Essay template (စာစီစာကုံး / အက်ဆေး ပုံစံခွက်).
pub fn myanmar_essay() -> Document {
    let mut d = Document::new();
    d.core.title = "စာဖတ်ခြင်းမှ ရရှိသော အသိပညာနှင့် အတွေးအခေါ်".into();
    d.core.creator = "HtetWord".into();
    let b: Vec<Block> = vec![
        Block::Para(styled("စာဖတ်ခြင်းမှ ရရှိသော အသိပညာနှင့် အတွေးအခေါ်", "Title")),
        Block::Para(styled("အထက်တန်းနှင့် တက္ကသိုလ်အဆင့် စာစီစာကုံး ပုံစံခွက်", "Subtitle")),
        Block::Para(para("အမည် - မောင်မြတ်မင်းခေါင်")),
        Block::Para(styled("အဖွင့် အပိုဒ် (နိဒါန်း)", "Heading1")),
        Block::Para(para("လူတစ်ဦးတစ်ယောက်၏ အရည်အသွေးနှင့် အဆင့်အတန်းကို တိုင်းတာရာတွင် သူဖတ်ရှုသော စာပေအမျိုးအစားနှင့် အလေ့အထသည် အလွန်အရေးပါသော အခန်းကဏ္ဍမှ ပါဝင်ပါသည်။ စာအုပ်စာပေသည် လူသားတို့၏ မိတ်ဆွေစစ်ဖြစ်ပြီး ဘဝခရီးလမ်းတွင် အမှောင်ခွင်း၍ အလင်းဆောင်ပေးသော ဆီမီးတန်ဆောင်နှင့် တူပေသည်။")),
        Block::Para(styled("အလယ် အပိုဒ် (စာကိုယ်)", "Heading1")),
        Block::Para(para("စာပေဖတ်ရှုခြင်း၏ အကျိုးကျေးဇူးများကို လေ့လာကြည့်ပါက စိတ်ကူးဉာဏ်ကြွယ်ဝစေခြင်း၊ ဝေဖန်ပိုင်းခြားနိုင်သော ဉာဏ်ပညာ ဖွံ့ဖြိုးလာခြင်းနှင့် အမှားအမှန်ကို ဆင်ခြင်နိုင်စွမ်း တိုးတက်လာခြင်း စသည့် အကျိုးတရားများကို တွေ့ရှိရမည်ဖြစ်သည်။")),
        Block::Para(styled("အပိတ် အပိုဒ် (နိဂုံး)", "Heading1")),
        Block::Para(para("အချုပ်အားဖြင့်ဆိုသော် လူငယ်မောင်မယ်များသည် အချိန်ကို အကျိုးမဲ့ ကုန်လွန်စေခြင်းမရှိဘဲ မိမိတို့ဘဝ တိုးတက်မြင့်မားရေးအတွက် အဖိုးတန်သော စာအုပ်စာစောင်များကို မပြတ်လေ့လာ ဖတ်ရှုဆည်းပူးသွားသင့်ပေသည်။")),
    ];
    d.body = b.into_iter().map(std::sync::Arc::new).collect();
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn samples_are_valid() {
        assert_eq!(sample_document().word_count(), 361);
        for d in [
            sample_document(),
            letter(),
            resume(),
            report(),
            myanmar_article(),
            myanmar_book(),
            myanmar_post(),
            myanmar_essay(),
        ] {
            assert!(d.word_count() > 5);
            let l = wordcraft_layout::layout(&d, &mut wordcraft_layout::LayoutCache::new(), &Default::default());
            assert!(!l.pages.is_empty());
        }
    }
}
