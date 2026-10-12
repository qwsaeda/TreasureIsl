/* ea:   0x0003F8FC */
/* name: sub_3F8FC */
/* size: 488 */
wchar_t *__fastcall sub_3F8FC(wchar_t *a1, int a2, int a3, int a4, wchar_t *a5)
{
  int v8; // r0
  int v9; // r0
  int v10; // r0
  int v11; // r5
  void *v12; // r0
  void *v13; // r4
  wchar_t *v15; // [sp+4h] [bp-88h] BYREF
  void *v16[4]; // [sp+8h] [bp-84h] BYREF
  int (**v17)(); // [sp+18h] [bp-74h]
  void *v18[5]; // [sp+1Ch] [bp-70h] BYREF
  int (__fastcall **v19)(void *); // [sp+30h] [bp-5Ch] BYREF
  char v20; // [sp+34h] [bp-58h]
  int v21; // [sp+38h] [bp-54h]
  _DWORD v22[17]; // [sp+48h] [bp-44h] BYREF

  sub_43B8C(a1, a2);
  *((_DWORD *)a1 + 255) = a3;
  *((_DWORD *)a1 + 257) = a4 - 1;
  *((_DWORD *)a1 + 256) = 0;
  *(_DWORD *)a1 = &off_6F47C;
  *((_DWORD *)a1 + 522) = 0;
  *((_DWORD *)a1 + 523) = 0;
  memset(a1 + 516, 0, 0x400u);
  swprintf(a1 + 516, a5);
  *((_DWORD *)a1 + 514) = 0;
  *((_DWORD *)a1 + 515) = 0;
  *((_DWORD *)a1 + 516) = 0;
  *((_DWORD *)a1 + 517) = 0;
  *((_DWORD *)a1 + 518) = 0;
  *((_DWORD *)a1 + 519) = 0;
  *((_DWORD *)a1 + 520) = 0;
  *((_DWORD *)a1 + 521) = 0;
  *((_DWORD *)a1 + 514) = a1 + 516;
  sub_1333C(v16);
  sub_1309C((int)v16);
  v8 = sub_12F04(aDataGlpicsPak);
  sub_134A4((int)v16, (int)aDataGlpicsPak, (int)&aDataGlpicsPak[v8]);
  sub_1E61C((int)v22);
  sub_1DCEC((int)&v19);
  v19 = &off_6C938;
  sub_19490((int)v18);
  v18[3] = nullptr;
  v17 = &off_6C90C;
  if ( (unsigned __int8)sub_63BE8(v22, v16[0]) != 0 )
  {
    v15 = aSwitchpagePcx;
    v9 = sub_1E5CC(&v15);
    v10 = sub_1E84C((int)v22, v9);
    v11 = v10;
    if ( v10 == 0 )
      __debugbreak();
    v12 = operator new(*(_DWORD *)(v10 + 4));
    v13 = v12;
    if ( v12 == nullptr )
    {
      v17 = &off_6C90C;
      sub_194B4(v18);
      v19 = &off_6C910;
      sub_1E798(v22);
      sub_12F48(v16);
      return a1;
    }
    sub_63CE8((int)v22, v12);
    v21 = 16;
    v20 = 1;
    *((_DWORD *)a1 + 522) = sub_645B4(&v19, v13, *(_DWORD *)(v11 + 4));
    operator delete(v13);
  }
  v17 = &off_6C90C;
  sub_194B4(v18);
  v19 = &off_6C910;
  sub_1E798(v22);
  sub_12EA0((int)v16);
  return a1;
}

