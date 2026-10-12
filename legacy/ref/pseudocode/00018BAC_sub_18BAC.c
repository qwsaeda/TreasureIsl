/* ea:   0x00018BAC */
/* name: sub_18BAC */
/* size: 716 */
int __fastcall sub_18BAC(int a1, int a2)
{
  int v4; // r5
  int v6; // r4
  int v7; // r0
  void *v8; // r0
  void *v9; // r4
  _DWORD *v10; // r7
  int v11; // r1
  int v12; // r5
  int v13; // r0
  int v14; // r1
  int v15; // r5
  int v16; // r0
  char v17[4]; // [sp+4h] [bp-88h] BYREF
  int v18; // [sp+8h] [bp-84h] BYREF
  int v19; // [sp+Ch] [bp-80h] BYREF
  int (**v20)(); // [sp+14h] [bp-78h] BYREF
  _BYTE v21[12]; // [sp+18h] [bp-74h] BYREF
  int v22; // [sp+24h] [bp-68h]
  wchar_t v23[48]; // [sp+2Ch] [bp-60h] BYREF

  if ( a2 == 0 )
    __debugbreak();
  v4 = (*(int (__fastcall **)(int, wchar_t *, _DWORD, _DWORD))(*(_DWORD *)a2 + 8))(a2, aCardsCardsIni, 0, 0);
  if ( v4 == 0 )
    return 0;
  sub_19490(v21);
  v22 = 0;
  v20 = &off_6C90C;
  v6 = (*(int (__fastcall **)(int))(*(_DWORD *)v4 + 4))(v4);
  v7 = (*(int (__fastcall **)(int))(*(_DWORD *)v4 + 12))(v4);
  LOBYTE(v6) = sub_5F138(&v20, v7, v6);
  (*(void (__fastcall **)(int, int))(*(_DWORD *)a2 + 12))(a2, v4);
  if ( (_BYTE)v6 != 0 )
  {
    v8 = operator new(0xCu);
    v9 = v8;
    if ( v8 != nullptr )
      sub_19708(v8, v17);
    else
      v9 = nullptr;
    v10 = (_DWORD *)(a1 + 4);
    sub_19548(a1 + 4, v9);
    if ( (unsigned __int8)sub_5F498(&v20, 3667258, -1312219476, &v18) != 0 )
    {
      v11 = 0;
      if ( v18 <= 0 )
      {
LABEL_14:
        if ( (unsigned __int8)sub_5F498(&v20, 3667258, -922574397, &v18) != 0 )
        {
          v14 = 0;
          if ( v18 <= 0 )
          {
LABEL_24:
            sub_199D8(&v20, v14);
            return 1;
          }
          while ( 1 )
          {
            v15 = v14 + 1;
            swprintf(v23, aCardD, v14 + 1);
            v16 = sub_19A4C(v23);
            v19 = sub_18E78(a1, &v20, v16);
            if ( v19 == 0 )
              break;
            sub_195A4(a1 + 8, &v19);
            v14 = v15;
            if ( v15 >= v18 )
              goto LABEL_24;
          }
          if ( *v10 != 0 )
            sub_19630(*v10, 1);
        }
        else if ( *v10 != 0 )
        {
          sub_19630(*v10, 1);
        }
      }
      else
      {
        while ( 1 )
        {
          v12 = v11 + 1;
          swprintf(v23, aImageD, v11 + 1);
          v13 = sub_19A4C(v23);
          if ( (unsigned __int8)sub_192B4(a1, &v20, v13, *(_DWORD *)(a1 + 4), a2) == 0 )
            break;
          v11 = v12;
          if ( v12 >= v18 )
            goto LABEL_14;
        }
        if ( *v10 != 0 )
          sub_19630(*v10, 1);
      }
      *v10 = 0;
      v20 = &off_6C90C;
      sub_194B4(v21);
      return 0;
    }
  }
  v20 = &off_6C90C;
  sub_196AC(v21);
  sub_196F4(v21);
  return 0;
}

