Le but de cette plateforme est de permettre au bureau & aux 
responsables de l'IE de pouvoir gérer les redirections et liens 
sur le nom de domaine de isenengineering.fr.
*ex: discord.isenengineering.fr -> discord.gg/...*

Je le fais en rust pour le kiff et en plus ça forcera le prochain chargé de maintenance d'apprendre un nouveau langage, ça l'amusera...
puis ça contribura à la neutralité carbone (très efficace niveau mémoire et temps CPU).

## Structure
Front -> html + tailwindcss

Back -> rust
> - axum (serveur web)
> - rand 
> - serde
> - surrealdb (client pour la bdd de tide, si la dépendance devient trop embêtante on peut passer en REST)
> - tokio (async/await)
> - toml
> - tower-http
>
