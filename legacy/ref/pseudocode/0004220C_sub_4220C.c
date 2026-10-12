/* ea:   0x0004220C */
/* name: sub_4220C */
/* size: 364 */
int __fastcall sub_4220C(int *a1)
{
  int v2; // r0
  int (*v4)(); // r8
  int v5; // r4
  _DWORD *v6; // r6
  int v7; // r1
  int v8; // r3
  int v9; // r2
  int v10; // r1
  int v11; // r4
  _DWORD v12[4]; // [sp+Ch] [bp-50h] BYREF
  _DWORD v13[4]; // [sp+1Ch] [bp-40h] BYREF
  _DWORD v14[12]; // [sp+2Ch] [bp-30h] BYREF

  v2 = a1[12];
  if ( v2 <= 72 )
  {
    a1[12] = v2 + 1;
    v12[1] = 176;
    v4 = nullptr;
    v5 = a1[2];
    a1[3] = (int)sub_5B0CC;
    v12[3] = 220;
    v12[0] = 0;
    v12[2] = 0;
    (*(void (__fastcall **)(int, _DWORD *, int, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v5 + 20))(
      v5,
      v12,
      a1[13],
      v12,
      sub_5B0CC,
      &dword_81230,
      1);
    v6 = (_DWORD *)a1[8];
    v7 = *v6;
    if ( *v6 == 0 )
      __debugbreak();
    v8 = *(_DWORD *)(*v6 + 4);
    if ( v7 == 0 )
      __debugbreak();
    v9 = *(_DWORD *)(v7 + 8);
    v13[0] = 0;
    v13[1] = v8;
    v13[2] = 0;
    v13[3] = v9;
    if ( *v6 == 0 )
      __debugbreak();
    v10 = *(_DWORD *)(*v6 + 12);
    if ( v10 == 16 )
    {
      v4 = sub_5BC14;
    }
    else if ( v10 == 8 )
    {
      v4 = sub_5D9C4;
    }
    a1[3] = (int)v4;
    v14[0] = 34;
    v11 = a1[2];
    v14[1] = v8 + 34;
    v14[2] = 80;
    v14[3] = v9 + 80;
    return (*(int (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(*(_DWORD *)v11 + 20))(
             v11,
             v14,
             *v6,
             v13,
             v4,
             &dword_81230,
             1);
  }
  else
  {
    sub_17C7C(a1[1], 0);
    return sub_3DC04(a1[1], 4);
  }
}

