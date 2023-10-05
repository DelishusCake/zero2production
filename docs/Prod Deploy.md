How to deploy to prod
---

  1. Install the digitalocean cli
  2. Run the script below: 

```bash
doctl apps create --spec spec.yaml
DATABASE_URL=DIGIALOCEAN-DB-CONECTION-STRING sqlx migrate run
```