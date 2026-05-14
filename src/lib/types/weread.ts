export interface WereadAccount {
  id: number;
  vid: string;
  username: string;
  avatarUrl: string | null;
  cookiesJson: string;
  cookieExpiresAt: number | null;
  isActive: number;
  createdAt: number;
  updatedAt: number;
}

export interface WereadBook {
  id: number;
  libraryId: number;
  bookId: string;
  title: string;
  author: string | null;
  coverUrl: string | null;
  intro: string | null;
  category: string | null;
  wordCount: number;
  chapterCount: number;
  createdAt: number;
}

export interface WereadChapter {
  id: number;
  libraryId: number;
  bookId: number;
  chapterUid: string;
  title: string;
  level: number;
  wordCount: number;
  sortOrder: number;
  contentMd: string | null;
  extractedAt: number | null;
}

export interface WereadDownload {
  id: number;
  libraryId: number;
  bookId: number;
  status: string;
  progressCurrent: number;
  progressTotal: number;
  outputPath: string | null;
  errorMessage: string | null;
  startedAt: number | null;
  completedAt: number | null;
  createdAt: number;
}

export interface LoginStatus {
  loggedIn: boolean;
  account: WereadAccount | null;
}

export interface ExportProgress {
  bookId: number;
  title: string;
  current: number;
  total: number;
  status: string;
}
