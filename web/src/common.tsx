export const Loader = () => (<div className="loader">
    <img src="/loader.svg" alt="Loading" />
    <h3>Loading...</h3>
</div>);

export const ErrMsg = ({ msg }: { msg?: string }) =>
(<div className="error">
    <h1 className="sadEmoji">🥺</h1>
    <h3>Something went wrong!</h3>
    {msg != null && <h4>{msg}</h4>}
</div>);