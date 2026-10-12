/* ea:   0x0002FC44 */
/* name: sub_2FC44 */
/* size: 916 */
int __fastcall sub_2FC44(int a1, int a2)
{
  int v4; // r0
  int v5; // r0
  int v6; // r4
  void *v7; // r0
  void *v8; // r5
  int v9; // r5
  int v10; // r0
  void *v11; // r0
  signed int v13; // r7
  int v14; // r4
  int v15; // r0
  int v16; // r11
  int *v17; // r10
  int v18; // r4
  int v19; // r0
  int v20; // r0
  int v21; // r0
  int v22; // r7
  void *v23; // r0
  void *v24; // r4
  int v25; // r0
  int v26; // r7
  int v27; // r0
  signed int v28; // [sp+0h] [bp-C0h] BYREF
  void *v29; // [sp+4h] [bp-BCh]
  int (**v30)(); // [sp+8h] [bp-B8h] BYREF
  void *v31[5]; // [sp+Ch] [bp-B4h] BYREF
  void *v32[3]; // [sp+20h] [bp-A0h] BYREF
  signed int v33; // [sp+2Ch] [bp-94h]
  int v34; // [sp+30h] [bp-90h]
  wchar_t *v35; // [sp+34h] [bp-8Ch] BYREF
  void *v36; // [sp+38h] [bp-88h]
  _WORD *v37; // [sp+3Ch] [bp-84h] BYREF
  int (__fastcall **v38)(void *); // [sp+40h] [bp-80h] BYREF
  char v39; // [sp+44h] [bp-7Ch]
  int v40; // [sp+48h] [bp-78h]
  _DWORD v41[12]; // [sp+58h] [bp-68h] BYREF
  wchar_t v42[28]; // [sp+88h] [bp-38h] BYREF

  v34 = a1;
  sub_1E61C((int)v41);
  sub_1DCEC((int)&v38);
  v38 = &off_6C938;
  sub_19490((int)v31);
  v31[3] = nullptr;
  v30 = &off_6C90C;
  sub_1333C(v32);
  if ( (unsigned __int8)sub_63BE8(v41, a2) != 0 )
  {
    v35 = aMapinfoIni;
    v4 = sub_1E5CC(&v35);
    v5 = sub_1E84C((int)v41, v4);
    v6 = v5;
    if ( v5 == 0 )
      __debugbreak();
    v7 = operator new(*(_DWORD *)(v5 + 4));
    v8 = v7;
    v36 = v7;
    if ( v7 == nullptr )
      goto LABEL_7;
    sub_63CE8((int)v41, v7);
    sub_5F138(&v30, v8, *(_DWORD *)(v6 + 4));
    v9 = sub_19A4C(aFilenum);
    v10 = sub_19A4C(aFilenum);
    sub_5F498(&v30, v10, v9, &v28);
    v29 = nullptr;
    sub_1DDA4(a1 + 20, v28);
    v29 = operator new(4 * v28);
    if ( v29 == nullptr || (v11 = operator new(4 * v28), *(_DWORD *)(a1 + 44) = v11, v11 == nullptr) )
    {
LABEL_7:
      sub_12F48(v32);
      v30 = &off_6C90C;
      sub_194B4(v31);
      v38 = &off_6C910;
      sub_1E798(v41);
      return 0;
    }
    v13 = 0;
    if ( v28 > 0 )
    {
      do
      {
        v33 = v13 + 1;
        swprintf(v42, aSD, &unk_81180, v13 + 1);
        v14 = sub_19A4C(aAttrib);
        v15 = sub_19A4C(v42);
        v16 = 4 * v13;
        v17 = (int *)((char *)v29 + 4 * v13);
        sub_5F498(&v30, v15, v14, v17);
        v18 = sub_19A4C(aFilename);
        v19 = sub_19A4C(v42);
        sub_5F55C(&v30, v19, v18, v32);
        v37 = v32[0];
        v20 = sub_1E5CC(&v37);
        v21 = sub_1E84C((int)v41, v20);
        v22 = v21;
        if ( v21 == 0 )
          __debugbreak();
        v23 = operator new(*(_DWORD *)(v21 + 4));
        v24 = v23;
        if ( v23 == nullptr )
          goto LABEL_7;
        sub_63CE8((int)v41, v23);
        v40 = 16;
        v39 = 1;
        v25 = sub_645B4(&v38, v24, *(_DWORD *)(v22 + 4));
        v26 = v34;
        *(_DWORD *)(v16 + *(_DWORD *)(v34 + 20)) = v25;
        operator delete(v24);
        v27 = *(_DWORD *)(v26 + 44);
        if ( *v17 <= 2 )
          *(_DWORD *)(v16 + v27) = 1;
        else
          sub_2FFD8(&v30, v42, aNum, v16 + v27);
        v13 = v33;
      }
      while ( v33 < v28 );
    }
    operator delete(v36);
    operator delete(v29);
    sub_12EA0((int)v32);
    sub_199D8(&v30);
    sub_1EA14(&v38);
    sub_1E798(v41);
    return 1;
  }
  else
  {
    sub_12EA0((int)v32);
    sub_199D8(&v30);
    sub_1EA14(&v38);
    sub_1E798(v41);
    return 0;
  }
}

