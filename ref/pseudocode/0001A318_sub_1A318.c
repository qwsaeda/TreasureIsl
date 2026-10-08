/* ea:   0x0001A318 */
/* name: sub_1A318 */
/* size: 296 */
int __fastcall sub_1A318(_DWORD *a1)
{
  _DWORD *v1; // lr
  int v2; // r8
  int v3; // r11
  _DWORD *v4; // r10
  int v5; // r0
  int v6; // r1
  int (*v7)(); // r5
  int v8; // r1
  int v9; // r3
  int v10; // r4
  _DWORD *v11; // r4
  int v12; // r5
  int result; // r0
  _DWORD v15[13]; // [sp+14h] [bp-34h] BYREF

  v1 = a1;
  v2 = a1[258];
  v3 = a1[259];
  if ( v2 != v3 )
  {
    v4 = a1;
    do
    {
      v5 = *(_DWORD *)(v2 + 8);
      if ( v5 == 0 )
        __debugbreak();
      v6 = *(_DWORD *)(v5 + 12);
      if ( v6 == 16 )
      {
        v7 = sub_5BC14;
      }
      else if ( v6 == 8 )
      {
        v7 = sub_5D9C4;
      }
      else
      {
        v7 = nullptr;
      }
      v1[122] = v7;
      v8 = v4[264];
      v9 = v4[265];
      v10 = v1[1];
      v15[1] = v8 + 50;
      v15[0] = v8;
      v15[2] = v9;
      v15[3] = v9 + 40;
      (*(void (__fastcall **)(int, _DWORD *, _DWORD, int, int (*)(), int *, int))(*(_DWORD *)v10 + 20))(
        v10,
        v15,
        *(_DWORD *)(v2 + 8),
        v2 + 12,
        v7,
        &dword_81230,
        1);
      v2 += 36;
      v1 = a1;
      v4 += 2;
    }
    while ( v2 != v3 );
  }
  v11 = v1;
  v12 = 10;
  while ( 1 )
  {
    *(_DWORD *)(v1[284] + 12) = v11[264];
    *(_DWORD *)(v1[284] + 16) = v11[265];
    result = sub_460F8(v1[284], v1[1], v1[122]);
    --v12;
    v11 += 2;
    if ( v12 == 0 )
      break;
    v1 = a1;
  }
  return result;
}

