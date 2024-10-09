# http_server
Lightweight async HTTP server

## How to use

```shell
cargo run --example tradingview

curl http://localhost:3000/scrape --verbose -X POST -H 'Content-Type: application/json' -d '{
    "name": "client",
    "auth_token": "{{your_auth_token}}",
    "chart_symbols": [
        "={\\\"adjustment\\\":\\\"splits\\\",\\\"currency-id\\\":\\\"USD\\\",\\\"session\\\":\\\"regular\\\",\\\"symbol\\\":\\\"AMEX:SPY\\\"}"
    ],
    "quote_symbols": [
        "={\\\"adjustment\\\":\\\"splits\\\",\\\"currency-id\\\":\\\"USD\\\",\\\"session\\\":\\\"regular\\\",\\\"symbol\\\":\\\"AMEX:SPY\\\"}"
    ],
    "indicators": [
        "{\"text\":\"bmI9Ks46_14Oy1AFtjg8Ls9wU0S1rlg==_u70xwiBAuvwE8ScMuj3/xelBeUlPpaP443vgI0LOz0anO3Sz0Nml/Cw66rceMmOX/36sFmV/J8A9ocybTXK65SWNk5Mq5ULJ6IYlXtaoFYYsZRWpEMmaP9eq8c+j6BmHYcbh3XLrcNMUimL3emFm7ualhqyIU9Bit+n31nA898zBRSxB1+Jj5sHZ5cCUltgwmiCmbV6WhQoR6fRTVK5DXvgazVghDGv9ZF18/TpaZAnipKAZ1P59oNNL2e72XZQXWzWZlAbu7CHAtjyLv5RmO9bMBdsr2+Icd5cmGy+inNgtM4++cecagL5owwZhZGA/GRPyZ8UtjuvJesqiGPH+yqQEWtyfCnCjpvTV+tpDCn2SKcSQZyA87pNzAIi6/pspgUb01Sf2+wiJY+HuXAMKZQQ9zgD7oIvjjPaQqTBUgjVc0VMlQYX98yW3jzdOkaRXjKHxqSn0MXodjEBr1wQvH8sUv8Pvrttgdb7LVh/NFH4z8sQMRK7U7HB08M277TrUkz5Lak1OArmJ5vGF36Ty+Cw7nF3T2/t+LHecLwbIAzrtxR85m0fHMsZwwfW8z71w6/PuQnSZnlinambAWGDzUOAcc9CcXj9LRHsi9/wjRecaws1CUt1t4DI3oYsdMBcoGdx79k2a5qJT3aAYgpa1GTY3saW3RK5Lf8DasNK3srIlE6NyomS+pGhpBUpEFbd6iZL5o9G3iPUMHApZF3wXAHq78WxT+dnPUc/x3nnTmUK4IzsJnURj7jdi2Ko3LlC6OIO8o9/6knQPipTK7MMPG+sSJoFrfVaQiH6aXUMiTAspzHVmeoxZRFoi3J95HfXh+bOMbIwP62VmHgH0RhZzHWpUxIJof4iK/SIo3JVAQkt43JGyD8A0CzIgH2MVZmMV+rwe6URDCO63Vrs/6Fvz6QzPWbUmiXW5laTpBXJzM5mBrZD+M9Zso42rATUT6w3i23H2VE5kKbHG5p5kkyGM1c134cike1y5gyZDK3SMmnQyNgxUJKG0UpgXF2dnlQJpHXzya8dXco5QhldBd7TG33vKdKN5Ti/LMP6GJsZt6QC4CZWj0tWC8ow9ETVkiw0GGSLNUq818rG0EnWt9ZPVPu2dyT3gP/ZamMmmrKRWne12psNknznrqiH1ffDxdGGkJgVpda377gPVPYK5XrzyXvQKhNf7/xdAqN5DAiW5xpiUJ6GFcl3sgR35OBsFkFA=\",\"pineId\":\"PUB;N16MOYK6AEJGGAoy40axs0S48GRFYcNn\",\"pineVersion\":\"1.0\",\"in_0\":{\"v\":1,\"f\":true,\"t\":\"integer\"},\"in_1\":{\"v\":\"close\",\"f\":true,\"t\":\"source\"},\"in_2\":{\"v\":7,\"f\":true,\"t\":\"integer\"},\"in_3\":{\"v\":\"close\",\"f\":true,\"t\":\"source\"},\"in_4\":{\"v\":25,\"f\":true,\"t\":\"integer\"},\"in_5\":{\"v\":65,\"f\":true,\"t\":\"integer\"},\"in_6\":{\"v\":51,\"f\":true,\"t\":\"integer\"},\"in_7\":{\"v\":21,\"f\":true,\"t\":\"integer\"}}"
    ],
    "timeframe": "5",
    "range": 300,
    "mode": "Standard"
}' 
```
