<footer class="site-footer">
  <div class="footer-inner">
    <span class="footer-copyright">
      &copy; {new Date().getFullYear()} Thorben Woelk. MIT License.
    </span>
    <a
      href="https://github.com/ThorbenWoelk/dAstIll"
      target="_blank"
      rel="noopener noreferrer"
      class="footer-link"
    >
      <svg
        width="16"
        height="16"
        viewBox="0 0 24 24"
        fill="currentColor"
        aria-hidden="true"
      >
        <path
          d="M12 0C5.37 0 0 5.37 0 12c0 5.31 3.435 9.795 8.205 11.385.6.105.825-.255.825-.57 0-.285-.015-1.23-.015-2.235-3.015.555-3.795-.735-4.035-1.41-.135-.345-.72-1.41-1.23-1.695-.42-.225-1.02-.78-.015-.795.945-.015 1.62.87 1.845 1.23 1.08 1.815 2.805 1.305 3.495.99.105-.78.42-1.305.765-1.605-2.67-.3-5.46-1.335-5.46-5.925 0-1.305.465-2.385 1.23-3.225-.12-.3-.54-1.53.12-3.18 0 0 1.005-.315 3.3 1.23.96-.27 1.98-.405 3-.405s2.04.135 3 .405c2.295-1.56 3.3-1.23 3.3-1.23.66 1.65.24 2.88.12 3.18.765.84 1.23 1.905 1.23 3.225 0 4.605-2.805 5.625-5.475 5.925.435.375.81 1.095.81 2.22 0 1.605-.015 2.895-.015 3.3 0 .315.225.69.825.57A12.02 12.02 0 0 0 24 12c0-6.63-5.37-12-12-12z"
        />
      </svg>
      <span>GitHub</span>
    </a>
  </div>
</footer>

<style>
  .site-footer {
    width: 100%;
    border-top: 1px solid var(--border-soft);
    background: var(--background);
  }

  .footer-inner {
    max-width: 1440px;
    margin: 0 auto;
    padding: 1rem 1.5rem;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .footer-copyright {
    font-size: 0.75rem;
    color: var(--soft-foreground);
    opacity: 0.6;
    letter-spacing: 0.01em;
  }

  .footer-link {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    font-size: 0.75rem;
    font-weight: 600;
    color: var(--soft-foreground);
    opacity: 0.5;
    text-decoration: none;
    transition:
      opacity 200ms ease,
      color 200ms ease;
  }

  .footer-link:hover {
    opacity: 1;
    color: var(--foreground);
  }

  .footer-link:focus-visible {
    outline: 2px solid var(--accent);
    outline-offset: 2px;
    border-radius: 4px;
  }

  @media (max-width: 1023px) {
    .site-footer {
      padding-bottom: calc(3.5rem + env(safe-area-inset-bottom) + 0.25rem);
    }
  }
</style>
