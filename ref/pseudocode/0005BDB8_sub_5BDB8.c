/* ea:   0x0005BDB8 */
/* name: sub_5BDB8 */
/* size: 440 */
int __fastcall sub_5BDB8(_DWORD *a1, int a2, int a3)
{
  int v6; // r0
  int v7; // r1
  bool v8; // zf
  int result; // r0
  int v10; // r0
  unsigned __int16 *v11; // r10
  int v12; // r4
  int v13; // r11
  int v14; // r9
  int v15; // r1
  int v16; // r3
  int v17; // r4
  int v18; // r3
  int v19; // r6
  unsigned __int16 *v20; // r7
  int v21; // r2
  unsigned __int16 v22; // r1
  unsigned __int16 v23; // t1
  char v24; // r0
  unsigned __int16 v25; // [sp+0h] [bp-64h]
  _DWORD v28[4]; // [sp+10h] [bp-54h] BYREF
  _DWORD v29[4]; // [sp+20h] [bp-44h] BYREF
  int v30; // [sp+30h] [bp-34h]
  int v31; // [sp+34h] [bp-30h]
  int v32; // [sp+38h] [bp-2Ch]
  int v33; // [sp+3Ch] [bp-28h]

  if ( a1 == nullptr || a1[3] != 16 )
    __debugbreak();
  if ( a3 == 0 || *(_DWORD *)(a3 + 12) != 16 )
    __debugbreak();
  if ( a1 == nullptr )
    __debugbreak();
  v6 = a1[1];
  v7 = a1[2];
  v28[0] = 0;
  v28[1] = v6;
  v28[2] = 0;
  v28[3] = v7;
  sub_5E9E0(v29, v28, a2);
  v8 = v30 == v31;
  if ( v30 == v31 )
    v8 = v33 == v32;
  LOBYTE(result) = v8;
  result = (unsigned __int8)result;
  if ( (_BYTE)result == 0 )
  {
    v10 = sub_5E970(a1, v32);
    v11 = (unsigned __int16 *)(v10 + 2 * v30);
    v12 = v31 - v30;
    v13 = v33 - v32;
    if ( a3 == 0 )
      __debugbreak();
    v14 = sub_5E970(a3, v29[2]);
    if ( a3 == 0 )
      __debugbreak();
    v15 = *(_DWORD *)(a3 + 4);
    result = v29[0];
    v16 = v15 - v29[0] - v12;
    if ( v13 != 0 )
    {
      v17 = 2 * (v15 - v29[0]);
      v18 = 2 * v16;
      do
      {
        v19 = v17 + v14;
        v20 = v11;
        if ( v18 + v14 != v17 + v14 )
        {
          v21 = (unsigned __int16)word_8122C;
          do
          {
            v23 = *(_WORD *)(v19 - 2);
            v19 -= 2;
            v22 = v23;
            v25 = v23;
            if ( v21 == v23 )
            {
              v24 = 0;
              v22 = v25;
            }
            else
            {
              v24 = 1;
            }
            if ( v24 != 0 )
              *v20 = v22;
            ++v20;
          }
          while ( v19 != v18 + v14 );
        }
        --v13;
        result = a1[1];
        v11 += result;
        v14 += 2 * *(_DWORD *)(a3 + 4);
      }
      while ( v13 != 0 );
    }
  }
  return result;
}

