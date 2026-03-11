<script lang="ts">
	import { onMount } from 'svelte';
	import { page } from '$app/stores';
	import ChapterList from '$lib/components/ChapterList.svelte';
	import Reader from '$lib/components/Reader.svelte';

	interface Chapter {
		id: string;
		order: number;
		title: string;
		wordCount: number;
	}

	interface Book {
		id: string;
		title: string;
		author: string;
		directory: string;
	}

	let book = $state<Book | null>(null);
	let chapters = $state<Chapter[]>([]);
	let currentChapter = $state<Chapter | null>(null);
	let bookId = $page.params.bookId;

	function loadBook() {
		// TODO: Replace with API call to fetch book details from backend
		book = {
			id: bookId,
			title: 'The Great Adventure',
			author: 'John Smith',
			directory: `/books/${bookId}`
		};
	}

	function loadChapters() {
		// TODO: Replace with API call to fetch chapters from backend
		chapters = [
			{
				id: 'ch1',
				order: 1,
				title: 'Chapter 1: The Beginning',
				wordCount: 2500
			},
			{
				id: 'ch2',
				order: 2,
				title: 'Chapter 2: Discovery',
				wordCount: 3200
			},
			{
				id: 'ch3',
				order: 3,
				title: 'Chapter 3: The Journey',
				wordCount: 2800
			},
			{
				id: 'ch4',
				order: 4,
				title: 'Chapter 4: Challenges',
				wordCount: 3100
			},
			{
				id: 'ch5',
				order: 5,
				title: 'Chapter 5: Resolution',
				wordCount: 2900
			}
		];

		if (chapters.length > 0) {
			currentChapter = chapters[0];
		}
	}

	function handleSelectChapter(chapterId: string) {
		const selected = chapters.find((ch) => ch.id === chapterId);
		if (selected) {
			currentChapter = selected;
		}
	}

	onMount(() => {
		loadBook();
		loadChapters();
	});
</script>

<div class="reader-page">
	<ChapterList
		{chapters}
		currentChapterId={currentChapter?.id ?? null}
		onSelectChapter={handleSelectChapter}
	/>
	<Reader chapter={currentChapter} bookDir={book?.directory ?? ''} />
</div>

<style>
	.reader-page {
		display: flex;
		width: 100%;
		height: 100%;
		overflow: hidden;
	}
</style>
