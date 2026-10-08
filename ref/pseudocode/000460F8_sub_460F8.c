/* ea:   0x000460F8 */
/* name: sub_460F8 */
/* size: 412 */
int __fastcall sub_460F8(_DWORD *a1, int *a2, int (*a3)())
{
  int result; // r0
  int v7; // r2
  int v8; // r1
  int v9; // r1
  int v10; // r1
  int v11; // r1
  int v12; // r5
  int v13; // r4
  int v14; // r3
  int v15; // r2
  int v16; // r1
  int v17; // r0
  int v18; // r4
  int v19; // r3
  int v20; // r0
  int v21; // r0
  _DWORD v22[4]; // [sp+Ch] [bp-3Ch] BYREF
  _DWORD v23[11]; // [sp+1Ch] [bp-2Ch] BYREF

  result = a1[5];
  if ( result != 0 )
  {
    v7 = a1[7];
    if ( v7 == 1 )
    {
      if ( *a1 == 0 )
        __debugbreak();
      v8 = *(_DWORD *)(*a1 + 12);
      if ( v8 == 16 )
      {
        a3 = sub_5BC14;
      }
      else if ( v8 == 8 )
      {
        a3 = sub_5D9C4;
      }
      else
      {
        a3 = nullptr;
      }
    }
    if ( v7 == 2 )
    {
      if ( *a1 == 0 )
        __debugbreak();
      v9 = *(_DWORD *)(*a1 + 12);
      if ( v9 == 16 )
      {
        a3 = sub_5BF70;
      }
      else if ( v9 == 8 )
      {
        a3 = sub_5E158;
      }
      else
      {
        a3 = nullptr;
      }
    }
    if ( v7 == 3 )
    {
      if ( *a1 == 0 )
        __debugbreak();
      v10 = *(_DWORD *)(*a1 + 12);
      if ( v10 == 16 )
      {
        a3 = sub_5BDB8;
      }
      else if ( v10 == 8 )
      {
        a3 = sub_5DD7C;
      }
      else
      {
        a3 = nullptr;
      }
    }
    v11 = a1[11];
    v12 = a1[10];
    v13 = a1[9];
    v14 = (v11 - 1) * v12;
    v22[0] = 0;
    v15 = v12 * v11;
    v16 = a1[3];
    v22[1] = v13;
    v17 = v16 + v13;
    v18 = *a2;
    v22[2] = v14;
    v19 = a1[4];
    v22[3] = v15;
    v23[1] = v17;
    v23[3] = v19 + v12;
    v23[0] = v16;
    v23[2] = v19;
    result = (*(int (__fastcall **)(int *, _DWORD *, _DWORD, _DWORD *, int (*)(), int *, int))(v18 + 20))(
               a2,
               v23,
               *a1,
               v22,
               a3,
               &dword_81230,
               1);
    if ( a1[8] == 0 )
    {
      v20 = a1[12];
      a1[12] = v20 + 1;
      if ( v20 == 9 )
      {
        v21 = a1[11];
        a1[12] = 0;
        a1[11] = v21 + 1;
      }
      result = a1[1];
      if ( a1[11] > result )
        a1[11] = 1;
    }
  }
  return result;
}

