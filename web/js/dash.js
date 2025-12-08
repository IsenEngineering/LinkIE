const entries = []

class Entry {
    constructor(type, source, destination, id) {
        if(!['subdomain', 'path', 'both'].includes(type)) throw new Error("Wrong type")
        this.type = type
        this.destination = destination
        this.subdomain = source.subdomain || null
        this.path = source.path || null
        this.id = id || Math.floor(Math.random() * 10E8).toString(36)

        this.node = this.create_node()
        this.create_listener()

        document.getElementById('entries').append(this.node)
    }

    display_type() {
        switch(this.type) {
            case 'subdomain':
                return "Sous-domaine"
            case 'path':
                return "Chemin"
        }
        return "Sous-domaine et chemin"
    }

    create_listener() {
        const edit = this.node.querySelector('.edit')
        const del = this.node.querySelector('.delete')

        edit.addEventListener('click', () => {
            console.log('edit')
            Edition.open({
                id: this.id,
                type: this.type,
                subdomain: this.subdomain,
                path: this.path,
                destination: this.destination,
                cb: ({ id, type, subdomain, path, destination }) => {
                    if(id !== this.id) return

                    this.update_destination(destination)
                    this.type = type
                    this.subdomain = subdomain
                    this.path = path
                    this._update_title()

                    console.log('edited')
                }
            })
        })
        del.addEventListener('click', () => {
            const i = entries.findIndex(e => this.id === e.id)
            if(i >= 0) entries.splice(i, 1)
            this.remove()
        })
    }

    create_node() {
        const nav = document.createElement('div')
        nav.innerHTML = `
        <span class="bg-black/10 px-2 py-1 rounded ">
            ${ this.display_type() }
        </span>
        <svg width="24" height="24" viewBox="0 0 24 24" class="edit">
            <path d="M21.174 6.812a1 1 0 0 0-3.986-3.987L3.842 16.174a2 2 0 0 0-.5.83l-1.321 4.352a.5.5 0 0 0 .623.622l4.353-1.32a2 2 0 0 0 .83-.497z"/>
            <path d="m15 5 4 4"/>
        </svg>
        <svg width="24" height="24" viewBox="0 0 24 24" class="delete">
            <path d="M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6"/>
            <path d="M3 6h18"/>
            <path d="M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2"/>
        </svg>`

        const title = document.createElement('h3')
        if(this.type == "both" || this.type == "subdomain") 
            title.innerHTML += `<span>${this.subdomain}</span>.`

        title.innerHTML += 'isenengineering.fr'

        if(this.type == "both" || this.type == "path")
            title.innerHTML += `/<span>${this.path}</span>`

        const link = document.createElement('a')
        link.href = this.destination
        link.target = '_blank'
        link.innerHTML = `
            <svg width="24" height="24" viewBox="0 0 24 24">
                <path d="M15 3h6v6"/>
                <path d="M10 14 21 3"/>
                <path d="M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6"/>
            </svg>
            <span>${this.destination}</span>`

        const article = document.createElement('article')
        article.append(nav, title, link)

        return article
    }

    update_destination(destination) {
        this.destination = destination

        this.node.querySelector('a > span').innerText = destination
    }

    _update_title() {
        let title = ""
        if(this.type == "both" || this.type == "subdomain") 
            title += `<span>${this.subdomain}</span>.`

        title += "isenengineering.fr"

        if(this.type == "both" || this.type == "path")
            title += `/<span>${this.path}</span>`

        this.node.querySelector('h3').innerHTML = title
    }
    update_subdomain(subdomain) {
        this.subdomain = subdomain
        this._update_title()
    }
    update_path(path) {
        this.path = path
        this._update_title()
    }
    remove() {
        this.node.remove()
    }
}

class Edition {
    static open({ id, type, subdomain, path, destination, cb }) {
        this.id = id
        this.subdomain = subdomain
        this.path = path
        this.destination = destination
        this.change_type(type)
        this.node.querySelector('#type').value = this.type
        this.node.querySelector('#destination').value = this.destination
        this.cb = cb

        this.node.hidden = false
    }
    
    static close() {
        this.node.hidden = true
        this.cb = null
    }

    static change_type(new_type) {
        if(this.type === new_type) return
        this.type = new_type

        const source = document.querySelector('#source')
        source.innerHTML = "<p>Source</p>"
        let subdomain, path
        switch(this.type) {
            case 'subdomain':
                this.path = null
                source.innerHTML += `<div>
                    <input type="text" id="subdomain" placeholder="domaine" 
                        value="${ this.subdomain || '' }">
                    <span>.isenengineering.fr</span>
                </div>`
                subdomain = source.querySelector('#subdomain')
                subdomain.addEventListener('input', () => this.subdomain = subdomain.value)
                break;
            case 'path':
                this.subdomain = null
                source.innerHTML += `<div>
                    <span>isenengineering.fr/</span>
                    <input type="text" id="path" placeholder="chemin"
                        value="${ this.path || '' }">
                </div>`
                path = source.querySelector('#path')
                path.addEventListener('input', () => this.path = path.value)
                break;
            case 'both':
                source.innerHTML += `<div>
                    <input type="text" id="subdomain" placeholder="domaine"
                        value="${ this.subdomain || '' }">
                    <span>.isenengineering.fr/</span>
                    <input type="text" id="path" placeholder="chemin"
                        value="${ this.path || '' }">
                </div>`
                console.log(source.innerHTML)

                subdomain = source.querySelector('#subdomain')
                subdomain.addEventListener('input', () => this.subdomain = subdomain.value)
                path = source.querySelector('#path')
                path.addEventListener('input', () => this.path = path.value)
                break;
        }
    }

    static init() {
        this.node = document.getElementById('edit')

        this.node.addEventListener('click', ev => {
            if(ev.target == this.node) this.close()
        })

        this.node.querySelector('#close-edit')
            .addEventListener('click', () => this.close())

        const type_input = this.node.querySelector('#type')
        type_input.addEventListener('input', () => {
            this.change_type(type_input.value)
        })

        const destination = this.node.querySelector('#destination')
        destination.addEventListener('input', () => {
            this.destination = destination.value
        })

        this.node.querySelector('#submit-edit').addEventListener('click', async () => {
            if(this.cb) await this.cb({
                id: this.id,
                subdomain: this.subdomain,
                path: this.path,
                type: this.type,
                destination: this.destination
            })

            this.close()
        })
    }
}

Edition.init()

entries.push(new Entry('path', { path: "nani" }, 'https://google.com'))
entries.push(new Entry('subdomain', { subdomain: "discord" }, 'https://discord.gg'))
entries.push(new Entry('both', { subdomain: "discord", path: "nani" }, 'https://wg.ie'))
