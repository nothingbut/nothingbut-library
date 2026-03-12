// Category data loaded from bsconfig.json
export interface CategoryData {
  category: string;
  subcategories: string[];
}

export const CATEGORIES: CategoryData[] = [
  {
    category: "玄幻",
    subcategories: ["东方玄幻", "异世大陆", "王朝争霸", "高武世界"]
  },
  {
    category: "奇幻",
    subcategories: [
      "现代魔法",
      "剑与魔法",
      "史诗奇幻",
      "神秘幻想",
      "历史神话",
      "另类幻想"
    ]
  },
  {
    category: "武侠",
    subcategories: [
      "传统武侠",
      "武侠幻想",
      "国术无双",
      "古武未来",
      "武侠同人"
    ]
  },
  {
    category: "仙侠",
    subcategories: [
      "修真文明",
      "幻想修仙",
      "现代修真",
      "神话修真",
      "古典仙侠"
    ]
  },
  {
    category: "都市",
    subcategories: [
      "都市生活",
      "都市异能",
      "异术超能",
      "青春校园",
      "娱乐明星",
      "商战职场"
    ]
  },
  {
    category: "现实",
    subcategories: [
      "时代叙事",
      "家庭伦理",
      "女性题材",
      "青年故事",
      "社会悬疑",
      "人间百态"
    ]
  },
  {
    category: "军事",
    subcategories: [
      "军旅生涯",
      "军事战争",
      "战争幻想",
      "抗战烽火",
      "谍战特工"
    ]
  },
  {
    category: "历史",
    subcategories: [
      "架空历史",
      "秦汉三国",
      "上古先秦",
      "历史传记",
      "两晋隋唐",
      "五代十国",
      "两宋元明",
      "清史民国",
      "外国历史",
      "民间传说"
    ]
  },
  {
    category: "游戏",
    subcategories: [
      "电子竞技",
      "虚拟网游",
      "游戏异界",
      "游戏系统",
      "游戏主播"
    ]
  },
  {
    category: "体育",
    subcategories: ["篮球运动", "体育赛事", "足球运动", "其它运动"]
  },
  {
    category: "科幻",
    subcategories: [
      "古武机甲",
      "未来世界",
      "星际文明",
      "超级科技",
      "时空穿梭",
      "进化变异",
      "末世危机"
    ]
  },
  {
    category: "诸天无限",
    subcategories: ["无限", "诸天", "综漫"]
  },
  {
    category: "悬疑",
    subcategories: [
      "诡秘悬疑",
      "奇妙世界",
      "侦探推理",
      "探险生存",
      "古今传奇"
    ]
  },
  {
    category: "轻小说",
    subcategories: ["原生幻想", "恋爱日常", "衍生同人", "搞笑吐槽"]
  },
  {
    category: "女频",
    subcategories: [
      "古代言情",
      "仙侠奇缘",
      "现代言情",
      "浪漫青春",
      "玄幻言情",
      "悬疑推理",
      "科幻空间",
      "游戏竞技",
      "现实生活"
    ]
  }
];
