/* ea:   0x00043D94 */
/* name: sub_43D94 */
/* size: 624 */
void sub_43D94(int a1, ...)
{
  int v2; // r7
  void *v3; // r1
  int v4; // r0
  int v5; // r0
  int v6; // r5
  void *v7; // r4
  int v8; // r5
  int v9; // r0
  int v10; // r11
  int v11; // r10
  int v12; // r4
  int v13; // r0
  int v14; // r4
  int v15; // r0
  int v16; // r0
  int v17; // r0
  int v18; // r7
  void *v19; // r0
  void *v20; // r4
  int v21; // r0
  _WORD *v22; // [sp+0h] [bp-D0h] BYREF
  int (**v23)(); // [sp+8h] [bp-C8h] BYREF
  void *v24[4]; // [sp+Ch] [bp-C4h] BYREF
  int v25; // [sp+1Ch] [bp-B4h]
  int v26; // [sp+20h] [bp-B0h] BYREF
  wchar_t *v27; // [sp+24h] [bp-ACh] BYREF
  void *v28[4]; // [sp+28h] [bp-A8h] BYREF
  int (__fastcall **v29)(void *); // [sp+38h] [bp-98h] BYREF
  char v30; // [sp+3Ch] [bp-94h]
  char v31; // [sp+44h] [bp-8Ch]
  wchar_t v32[12]; // [sp+50h] [bp-80h] BYREF
  _DWORD v33[22]; // [sp+68h] [bp-68h] BYREF
  void *varg_r1; // [sp+C4h] [bp-Ch] BYREF
  va_list varg_r1a; // [sp+C4h] [bp-Ch]
  va_list varg_r2; // [sp+C8h] [bp-8h] BYREF

  va_start(varg_r2, a1);
  va_start(varg_r1a, a1);
  varg_r1 = va_arg(varg_r2, void *);
  v25 = a1;
  sub_1E61C((int)v33);
  v2 = 0;
  v29 = &off_6C938;
  v31 = 0;
  v30 = 0;
  sub_19490((int)v24);
  v24[3] = nullptr;
  v23 = &off_6C90C;
  sub_1333C(v28);
  v3 = varg_r1;
  *(_DWORD *)(a1 + 1012) = 1;
  if ( (unsigned __int8)sub_63BE8(v33, v3) != 0 )
  {
    v27 = aMapinfoIni;
    v4 = sub_1E5CC(&v27);
    v5 = sub_1E84C((int)v33, v4);
    v6 = v5;
    if ( v5 == 0 )
      __debugbreak();
    v7 = operator new(*(_DWORD *)(v5 + 4));
    sub_63CE8((int)v33, v7);
    sub_5F138(&v23, v7, *(_DWORD *)(v6 + 4));
    operator delete(v7);
    v8 = sub_19A4C(aFilenum);
    v9 = sub_19A4C(aFilenum);
    sub_5F498(&v23, v9, v8, a1 + 492);
    if ( *(int *)(a1 + 492) > 0 )
    {
      v10 = a1;
      do
      {
        v11 = v2 + 1;
        swprintf(v32, aSD, &unk_81180, v2 + 1);
        v12 = sub_19A4C(aFilename);
        v13 = sub_19A4C(v32);
        sub_5F55C(&v23, v13, v12, v28);
        v14 = sub_19A4C(aNum);
        v15 = sub_19A4C(v32);
        sub_5F498(&v23, v15, v14, &v26);
        v22 = v28[0];
        *(_DWORD *)(v10 + 496) = v26;
        v16 = sub_1E5CC(&v22);
        v17 = sub_1E84C((int)v33, v16);
        v18 = v17;
        if ( v17 == 0 )
          __debugbreak();
        v19 = operator new(*(_DWORD *)(v17 + 4));
        v20 = v19;
        if ( v19 != nullptr )
        {
          sub_63CE8((int)v33, v19);
          v21 = sub_645B4(&v29, v20, *(_DWORD *)(v18 + 4));
          *(_DWORD *)(v10 + 8) = v21;
          if ( v21 != 0 )
            operator delete(v20);
        }
        v2 = v11;
        v10 += 4;
      }
      while ( v11 < *(_DWORD *)(v25 + 492) );
    }
  }
  sub_132A0(v28);
  v23 = &off_6C90C;
  sub_196AC(v24);
  sub_196F4(v24);
  v29 = &off_6C910;
  sub_1E798(v33);
  sub_132A0((void **)varg_r1a);
}

