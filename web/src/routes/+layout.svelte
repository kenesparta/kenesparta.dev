<style>
    :global(*) {
        margin: 0;
        padding: 0;
        font-family: 'Petrona', serif;
    }
    :global(html), :global(body) {
        height: 100%;
        width: 100%;
    }

    :global(body) {
        -webkit-font-smoothing: antialiased;
        letter-spacing: 0.01em;
    }

    :global(:root) {
        --color-primary: #0092FA;
        --color-secondary: #4869CC;
        --color-text: #41444D;
        --color-accent: #1B1C21;
        --color-accent-sec: #5A6CA3;
    }
</style>

<slot/>
