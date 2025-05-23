<style>
    .grid-container {
        display: grid;
        grid-template-columns: 1fr 2fr 1fr;
        min-height: 100vh;
    }

    .sidebar {
        background-color: #fff;
        padding: 1rem;
    }

    .content {
        background-color: #fff;
        display: grid;
        padding: 1rem;
        place-items: center;
    }

    .main__section {
        display: grid;
        place-items: center;
        align-content: center;
    }

    .logo-container {
        background-color: var(--color-accent);
        padding: 1rem;
        border-radius: 10%;
        width: 70px;
        height: 70px;
        display: flex;
        align-items: center;
        justify-content: center;
    }

    .logo {
        background-image: url('/icon-wob.svg');
        background-repeat: no-repeat;
        background-position: center;
        background-size: contain;
        width: 100%;
        height: 100%;
    }

    .title {
        margin-top: .8rem;
        font-size: 2.3rem;
        font-weight: bolder;
        color: var(--color-primary);
    }

    .subtitle {
        margin-top: .5rem;
        font-size: 1.4rem;
        font-weight: bold;
        color: var(--color-accent);
    }

    .slogan {
        margin-top: 1.5rem;
        font-size: 1.3rem;
        font-style: oblique;
        background: linear-gradient(
                -30deg,
                var(--color-primary) 0%,
                var(--color-secondary) 50%,
                var(--color-accent) 100%
        );
        background-size: 200% auto;
        background-clip: text;
        -webkit-background-clip: text;
        color: transparent;
        letter-spacing: 0.02em;
        max-width: 500px;
        text-align: center;
        animation: gradient-shift 3s ease infinite;
    }

    @keyframes gradient-shift {
        0% {
            background-position: 0 50%;
        }
        50% {
            background-position: 100% 50%;
        }
        100% {
            background-position: 0 50%;
        }
    }
</style>

<div class="grid-container">
    <div class="sidebar"></div>

    <div class="content">
        <section class="main__section">
            <div class="logo-container">
                <div class="logo" role="img" aria-label="Ken Esparta logo"></div>
            </div>
            <h1 class="title">Ken Esparta</h1>
            <h2 class="subtitle">Software Engineer</h2>
            <p class="slogan">Developing software that scales with your ambition.</p>
        </section>
    </div>

    <div class="sidebar"></div>
</div>