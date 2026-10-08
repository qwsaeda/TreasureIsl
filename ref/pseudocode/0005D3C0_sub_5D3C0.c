/* ea:   0x0005D3C0 */
/* name: sub_5D3C0 */
/* size: 532 */
int __fastcall sub_5D3C0(_DWORD *a1, int a2, _DWORD *a3, int a4, int a5)
{
  int v7; // r1
  unsigned __int8 v8; // r4
  bool v9; // zf
  int result; // r0
  int v11; // r0
  _WORD *v12; // r4
  int v13; // r5
  int v14; // r11
  int v15; // r0
  int v16; // r9
  unsigned __int8 *v17; // r10
  _WORD *v18; // r5
  unsigned __int8 *i; // r6
  int v20; // t1
  int v21; // r5
  _WORD *j; // r6
  _WORD *v23; // [sp+4h] [bp-64h]
  int v24; // [sp+8h] [bp-60h]
  _DWORD v27[4]; // [sp+14h] [bp-54h] BYREF
  _DWORD v28[4]; // [sp+24h] [bp-44h] BYREF
  int v29; // [sp+34h] [bp-34h]
  int v30; // [sp+38h] [bp-30h]
  int v31; // [sp+3Ch] [bp-2Ch]
  int v32; // [sp+40h] [bp-28h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == nullptr || a3[3] != 8 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v7 = a1[2];
  v8 = 0;
  v27[1] = a1[1];
  v27[3] = v7;
  v27[0] = 0;
  v27[2] = 0;
  sub_5E9E0(v28, v27, a2);
  v9 = v29 == v30;
  if ( v29 == v30 )
    v9 = v32 == v31;
  if ( v9 )
    v8 = 1;
  result = v8;
  if ( v8 == 0 )
  {
    v11 = sub_5E970(a1, v31);
    v12 = (_WORD *)(v11 + 2 * v29);
    v13 = v32 - v31;
    v23 = v12;
    v24 = v32 - v31;
    v14 = v30 - v29;
    if ( a3 == nullptr )
      __debugbreak();
    v15 = sub_5E970(a3, a3[2] - v28[2] - 1);
    v16 = a3[8];
    v17 = (unsigned __int8 *)(v15 + v28[0]);
    result = sub_5E920(a3);
    if ( (_BYTE)result != 0 )
    {
      if ( v13 != 0 )
      {
        while ( 1 )
        {
          v18 = v12;
          for ( i = v17; i != &v17[v14]; ++v18 )
          {
            v20 = *i++;
            *v18 = *(_WORD *)(v16
                            + 2
                            * (*(unsigned __int8 (__fastcall **)(int, int, int, _DWORD))(*(_DWORD *)a5 + 4))(
                                a5,
                                v20,
                                v16,
                                (unsigned __int16)*v18));
          }
          result = (int)a3;
          v23 += a1[1];
          v9 = v24-- == 1;
          v17 -= a3[1];
          if ( v9 )
            break;
          v12 = v23;
        }
      }
    }
    else
    {
      v21 = v24;
      for ( j = v12; v21 != 0; v17 -= result )
      {
        sub_5EB58(v17, &v17[v14], j, v16, a5);
        --v21;
        result = a3[1];
        j += a1[1];
      }
    }
  }
  return result;
}

