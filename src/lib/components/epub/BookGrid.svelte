<script lang="ts">
  import type { EpubBook } from '$lib/types/epub';
  import { convertFileSrc } from '@tauri-apps/api/core';

  interface Props {
    books: EpubBook[];
    onSelect: (book: EpubBook) => void;
  }

  let { books, onSelect }: Props = $props();

  /**
   * Convert a file path to a usable Tauri asset URL
   * Falls back to placeholder if cover is not available
   */
  function getCoverUrl(book: EpubBook): string {
    if (book.cover_path) {
      try {
        return convertFileSrc(book.cover_path);
      } catch (e) {
        console.warn(`Failed to convert cover path for book ${book.id}:`, e);
        return '/placeholder-cover.svg';
      }
    }
    return '/placeholder-cover.svg';
  }

  /**
   * Format author information
   * Note: Authors are not directly available in EpubBook type.
   * This is a placeholder that will be populated once EpubBookWithDetails is used.
   */
  function formatAuthors(book: EpubBook): string {
    // TODO: Get authors from EpubBookWithDetails when available
    // For now, return placeholder
    return 'Unknown Author';
  }

  /**
   * Convert rating number (0-5) to star representation
   */
  function getRatingStars(rating: number | null): string {
    if (!rating || rating < 0 || rating > 5) {
      return '';
    }
    const filledStars = Math.round(rating);
    const emptyStars = 5 - filledStars;
    return '★'.repeat(filledStars) + '☆'.repeat(emptyStars);
  }
</script>

<div class="grid grid-cols-2 gap-6 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 2xl:grid-cols-6">
  {#each books as book (book.id)}
    <button
      class="group cursor-pointer overflow-hidden rounded-lg bg-white shadow transition hover:shadow-lg"
      onclick={() => onSelect(book)}
      title={book.title}
    >
      <!-- Cover image -->
      <div class="aspect-[2/3] overflow-hidden bg-gray-200">
        <img
          src={getCoverUrl(book)}
          alt={book.title}
          class="h-full w-full object-cover transition group-hover:scale-105"
        />
      </div>

      <!-- Book information -->
      <div class="p-3">
        <!-- Title -->
        <h3 class="line-clamp-2 font-semibold text-gray-900" title={book.title}>
          {book.title}
        </h3>

        <!-- Authors -->
        <p class="mt-1 line-clamp-1 text-sm text-gray-600">
          {formatAuthors(book)}
        </p>

        <!-- Series information -->
        {#if book.series}
          <p class="mt-1 line-clamp-1 text-xs text-gray-500">
            {book.series}
            {#if book.series_index !== null}#{book.series_index}{/if}
          </p>
        {/if}

        <!-- Rating stars -->
        {#if book.rating}
          <div class="mt-2 text-sm text-yellow-500">
            {getRatingStars(book.rating)}
          </div>
        {/if}
      </div>
    </button>
  {/each}
</div>
