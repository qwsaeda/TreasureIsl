/* ea:   0x0005D9C4 */
/* name: sub_5D9C4 */
/* size: 492 */
int __fastcall sub_5D9C4(_DWORD *a1, int a2, _DWORD *a3)
{
  int v6; // r0
  int v7; // r1
  bool v8; // zf
  int result; // r0
  int v10; // r0
  _WORD *v11; // r8
  int v12; // r9
  int v13; // r7
  int v14; // r0
  unsigned __int8 *v15; // r5
  int v16; // r6
  _WORD *v17; // r3
  unsigned __int8 *i; // r2
  int v19; // r0
  int v20; // r1
  char v21; // r0
  __int16 v22; // r4
  __int16 v23; // [sp+4h] [bp-5Ch] BYREF
  int v24; // [sp+8h] [bp-58h]
  _DWORD v25[4]; // [sp+Ch] [bp-54h] BYREF
  _DWORD v26[4]; // [sp+1Ch] [bp-44h] BYREF
  int v27; // [sp+2Ch] [bp-34h]
  int v28; // [sp+30h] [bp-30h]
  int v29; // [sp+34h] [bp-2Ch]
  int v30; // [sp+38h] [bp-28h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == nullptr || a3[3] != 8 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v6 = a1[1];
  v7 = a1[2];
  v25[0] = 0;
  v25[1] = v6;
  v25[2] = 0;
  v25[3] = v7;
  sub_5E9E0(v26, v25, a2);
  v8 = v27 == v28;
  if ( v27 == v28 )
    v8 = v30 == v29;
  LOBYTE(result) = v8;
  result = (unsigned __int8)result;
  if ( (_BYTE)result == 0 )
  {
    v10 = sub_5E970(a1, v29);
    v11 = (_WORD *)(v10 + 2 * v27);
    v12 = v30 - v29;
    v13 = v28 - v27;
    v14 = sub_5E970(a3, v26[2]);
    v15 = (unsigned __int8 *)(v14 + v26[0]);
    if ( a3 == nullptr )
      __debugbreak();
    v16 = a3[8];
    result = sub_5E920(a3);
    if ( (_BYTE)result != 0 )
    {
      for ( ; v12 != 0; v15 += a3[1] )
      {
        v17 = v11;
        for ( i = v15; i != &v15[v13]; ++v17 )
        {
          v19 = *i++;
          v20 = v19;
          v24 = v19;
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
            *v17 = *(_WORD *)(v16 + 2 * v20);
        }
        result = a1[1];
        --v12;
        v11 += result;
      }
    }
    else if ( v12 != 0 )
    {
      LOWORD(v24) = (unsigned __int8)v23;
      v22 = (unsigned __int8)v23;
      do
      {
        v23 = v22;
        ((void (__fastcall *)(unsigned __int8 *, unsigned __int8 *, _WORD *, __int16 *, int))sub_5ECCC)(
          v15,
          &v15[v13],
          v11,
          &v23,
          v16);
        result = a3[1];
        --v12;
        v11 += a1[1];
        v15 += result;
      }
      while ( v12 != 0 );
    }
  }
  return result;
}

