import './Profile.css'
import logoSvg from '../assets/logo_no_bkg.svg';

function App() {
  return (
    <main className="min-h-screen grid place-items-center bg-custom">
      <div className="text-center max-w-2xl px-4 bg-cusom p-8">
        <img
          alt="Ken Esparta"
          src={logoSvg}
          className="mx-auto mb-10 w-56 h-56"
        />
        <h1 className="text-white text-3xl font-bold mb-10">Software Engineer</h1>
        <p className="text-xl text-white mb-4"> Hello! I'm Ken 👋 </p>
        <p className="text-xl text-white">
          I build secure, scalable APIs that convert your business requirements
          into value-driven solutions. Ready to learn more? Head to my blog. 🚀
        </p>
      </div>
    </main>
  );
}

export default App
