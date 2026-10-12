/* ea:   0x0005DD7C */
/* name: sub_5DD7C */
/* size: 508 */
int __fastcall sub_5DD7C(_DWORD *a1, int a2, int a3)
{
  int v5; // r0
  int v6; // r1
  bool v7; // zf
  int result; // r0
  int v9; // r0
  _WORD *v10; // r9
  int v11; // r10
  int v12; // r4
  int v13; // r8
  int v14; // r7
  int v15; // r5
  int v16; // r6
  int v17; // r2
  _WORD *i; // r3
  int v19; // t1
  int v20; // r1
  char v21; // r0
  __int16 v22; // r4
  __int16 v23; // [sp+4h] [bp-64h] BYREF
  int v24; // [sp+8h] [bp-60h]
  int v25; // [sp+Ch] [bp-5Ch]
  _DWORD v26[4]; // [sp+14h] [bp-54h] BYREF
  _DWORD v27[4]; // [sp+24h] [bp-44h] BYREF
  int v28; // [sp+34h] [bp-34h]
  int v29; // [sp+38h] [bp-30h]
  int v30; // [sp+3Ch] [bp-2Ch]
  int v31; // [sp+40h] [bp-28h]

  v24 = a3;
  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == 0 || *(_DWORD *)(a3 + 12) != 8 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v5 = a1[1];
  v6 = a1[2];
  v26[0] = 0;
  v26[1] = v5;
  v26[2] = 0;
  v26[3] = v6;
  sub_5E9E0(v27, v26, a2);
  v7 = v28 == v29;
  if ( v28 == v29 )
    v7 = v31 == v30;
  LOBYTE(result) = v7;
  result = (unsigned __int8)result;
  if ( (_BYTE)result == 0 )
  {
    v9 = sub_5E970(a1, v30);
    v10 = (_WORD *)(v9 + 2 * v28);
    v11 = v31 - v30;
    v12 = v29 - v28;
    v13 = sub_5E970(v24, v27[2]);
    if ( v24 == 0 )
      __debugbreak();
    v14 = *(_DWORD *)(v24 + 32);
    v15 = *(_DWORD *)(v24 + 4) - v27[0];
    v16 = v15 - v12;
    result = sub_5E920(v24);
    if ( (_BYTE)result != 0 )
    {
      for ( ; v11 != 0; v13 += *(_DWORD *)(v24 + 4) )
      {
        v17 = v15 + v13;
        for ( i = v10; v17 != v16 + v13; ++i )
        {
          v19 = *(unsigned __int8 *)--v17;
          v20 = v19;
          v25 = v19;
          if ( v19 != 0 )
          {
            v21 = 1;
          }
          else
          {
            v21 = 0;
            v20 = 0;
          }
          if ( v21 != 0 )
            *i = *(_WORD *)(v14 + 2 * v20);
        }
        --v11;
        v10 += a1[1];
        result = v24;
      }
    }
    else if ( v11 != 0 )
    {
      LOWORD(v25) = (unsigned __int8)v23;
      v22 = (unsigned __int8)v23;
      do
      {
        v23 = v22;
        ((void (__fastcall *)(int, int, _WORD *, __int16 *, int))sub_5EEA4)(v16 + v13, v15 + v13, v10, &v23, v14);
        --v11;
        result = *(_DWORD *)(v24 + 4);
        v10 += a1[1];
        v13 += result;
      }
      while ( v11 != 0 );
    }
  }
  return result;
}

