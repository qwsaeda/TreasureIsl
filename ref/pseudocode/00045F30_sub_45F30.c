/* ea:   0x00045F30 */
/* name: sub_45F30 */
/* size: 456 */
int __fastcall sub_45F30(_DWORD *a1, int a2, int *a3, int (__fastcall *a4)(int, int, int, int, int))
{
  int v7; // r0
  int result; // r0
  int v9; // r2
  int v10; // r1
  int v11; // r1
  int v12; // r1
  int v13; // r1
  int v14; // r5
  int v15; // r4
  int v16; // r3
  int v17; // r2
  int v18; // r1
  int v19; // r0
  int v20; // r4
  int v21; // r3
  int v22; // r0
  int v23; // r0
  int (**v24)(); // [sp+Ch] [bp-44h] BYREF
  int v25; // [sp+10h] [bp-40h]
  _DWORD v26[4]; // [sp+14h] [bp-3Ch] BYREF
  _DWORD v27[11]; // [sp+24h] [bp-2Ch] BYREF

  v25 = a2;
  v24 = off_6FDF0;
  if ( a2 == 64 )
  {
    v7 = 96;
  }
  else
  {
    if ( a2 != 96 )
      goto LABEL_6;
    v7 = 64;
  }
  v25 = v7;
LABEL_6:
  result = a1[5];
  if ( result != 0 )
  {
    v9 = a1[7];
    if ( v9 == 1 )
    {
      if ( *a1 == 0 )
        __debugbreak();
      v10 = *(_DWORD *)(*a1 + 12);
      if ( v10 == 16 )
      {
        a4 = sub_5C2CC;
      }
      else if ( v10 == 8 )
      {
        a4 = sub_5DBB0;
      }
      else
      {
        a4 = nullptr;
      }
    }
    if ( v9 == 2 )
    {
      if ( *a1 == 0 )
        __debugbreak();
      v11 = *(_DWORD *)(*a1 + 12);
      if ( v11 == 16 )
      {
        a4 = sub_5C6AC;
      }
      else if ( v11 == 8 )
      {
        a4 = sub_5E350;
      }
      else
      {
        a4 = nullptr;
      }
    }
    if ( v9 == 3 )
    {
      if ( *a1 == 0 )
        __debugbreak();
      v12 = *(_DWORD *)(*a1 + 12);
      if ( v12 == 16 )
      {
        a4 = sub_5C4BC;
      }
      else if ( v12 == 8 )
      {
        a4 = sub_5DF78;
      }
      else
      {
        a4 = nullptr;
      }
    }
    v13 = a1[11];
    v14 = a1[10];
    v15 = a1[9];
    v16 = (v13 - 1) * v14;
    v26[0] = 0;
    v17 = v14 * v13;
    v18 = a1[3];
    v26[1] = v15;
    v19 = v18 + v15;
    v20 = *a3;
    v26[2] = v16;
    v21 = a1[4];
    v26[3] = v17;
    v27[1] = v19;
    v27[3] = v21 + v14;
    v27[0] = v18;
    v27[2] = v21;
    result = (*(int (__fastcall **)(int *, _DWORD *, _DWORD, _DWORD *, int (__fastcall *)(int, int, int, int, int), int (***)(), int))(v20 + 20))(
               a3,
               v27,
               *a1,
               v26,
               a4,
               &v24,
               1);
    if ( a1[8] == 0 )
    {
      v22 = a1[12];
      a1[12] = v22 + 1;
      if ( v22 == 9 )
      {
        v23 = a1[11];
        a1[12] = 0;
        a1[11] = v23 + 1;
      }
      result = a1[1];
      if ( a1[11] > result )
      {
        a1[11] = 1;
        a1[8] = 1;
      }
    }
  }
  return result;
}

