/* ea:   0x0003CE74 */
/* name: sub_3CE74 */
/* size: 448 */
int __fastcall sub_3CE74(int a1, int a2)
{
  int v4; // r5
  int v5; // r4
  int v6; // r0
  int v8; // r1
  int v9; // r5
  int v10; // r0
  int v11; // [sp+0h] [bp-74h] BYREF
  int v12; // [sp+4h] [bp-70h] BYREF
  int (**v13)(); // [sp+8h] [bp-6Ch] BYREF
  void *v14[5]; // [sp+Ch] [bp-68h] BYREF
  wchar_t v15[42]; // [sp+20h] [bp-54h] BYREF

  if ( a2 == 0 )
    __debugbreak();
  v4 = (*(int (__fastcall **)(int, wchar_t *, _DWORD, _DWORD))(*(_DWORD *)a2 + 8))(a2, aCardsEventIni, 0, 0);
  if ( v4 == 0 )
    return 0;
  sub_3D3F4(&v13);
  v5 = (*(int (__fastcall **)(int))(*(_DWORD *)v4 + 4))(v4);
  v6 = (*(int (__fastcall **)(int))(*(_DWORD *)v4 + 12))(v4);
  LOBYTE(v5) = sub_5F138(&v13, v6, v5);
  (*(void (__fastcall **)(int, int))(*(_DWORD *)a2 + 12))(a2, v4);
  if ( (_BYTE)v5 == 0 )
  {
    v13 = &off_6C90C;
    sub_196AC(v14);
    sub_196F4(v14);
    return 0;
  }
  if ( (unsigned __int8)sub_5F498(&v13, 124132322, 121526345, &v11) == 0 )
  {
    sub_199D8(&v13);
    return 0;
  }
  v8 = 0;
  if ( v11 <= 0 )
  {
LABEL_12:
    v13 = &off_6C90C;
    sub_194B4(v14);
    return 1;
  }
  else
  {
    while ( 1 )
    {
      v9 = v8 + 1;
      swprintf(v15, aEventD, v8 + 1);
      v10 = sub_19A4C(v15);
      v12 = sub_3D034(a1, &v13, v10);
      if ( v12 == 0 )
        break;
      sub_3D1F8(a1 + 4, &v12);
      v8 = v9;
      if ( v9 >= v11 )
        goto LABEL_12;
    }
    v13 = &off_6C90C;
    sub_196AC(v14);
    sub_196F4(v14);
    return 0;
  }
}

