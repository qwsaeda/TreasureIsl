/* ea:   0x00012C50 */
/* name: sub_12C50 */
/* size: 592 */
int __fastcall sub_12C50(_DWORD *a1)
{
  _DWORD *v1; // r4
  int v2; // r11
  _DWORD *v3; // r10
  unsigned int v4; // r3
  int v5; // r0
  int (__fastcall *v6)(int, int, int, int, int); // r9
  int v7; // r7
  int v8; // r2
  int v9; // r0
  _DWORD *v10; // r0
  int v11; // r2
  int v12; // r4
  int v13; // r0
  int v14; // r3
  int v15; // r4
  int result; // r0
  int v17; // r0
  int v18; // r1
  _DWORD v20[4]; // [sp+14h] [bp-44h] BYREF
  _DWORD v21[13]; // [sp+24h] [bp-34h] BYREF

  v1 = a1;
  v2 = 0;
  v3 = a1;
  do
  {
    if ( v3[284] != 0 )
    {
      v4 = v1[283] + v2;
      if ( v4 <= 4 )
      {
        v5 = *(_DWORD *)(v3[284] + 12);
        if ( v5 == 16 )
        {
          v6 = sub_5C2CC;
        }
        else if ( v5 == 8 )
        {
          v6 = sub_5DBB0;
        }
        else
        {
          v6 = nullptr;
        }
        v1[122] = v6;
        v7 = v3[284];
        if ( v7 == 0 )
          __debugbreak();
        v8 = *(_DWORD *)(v3[284] + 8);
        if ( v7 == 0 )
          __debugbreak();
        v9 = *(_DWORD *)(v7 + 4);
        v20[0] = 0;
        v20[1] = v9;
        v20[2] = 0;
        v20[3] = v8;
        if ( v7 == 0 )
        {
          __debugbreak();
          __debugbreak();
        }
        v10 = &v1[2 * v4];
        v11 = v10[302];
        v12 = v10[303];
        v13 = *(_DWORD *)(v7 + 4);
        v14 = v12 + *(_DWORD *)(v7 + 8);
        v21[2] = v12;
        v21[0] = v11;
        v15 = a1[1];
        v21[1] = v11 + v13;
        v21[3] = v14;
        (*(void (__fastcall **)(int, _DWORD *, int, _DWORD *, int (__fastcall *)(int, int, int, int, int), int *, int))(*(_DWORD *)v15 + 20))(
          v15,
          v21,
          v7,
          v20,
          v6,
          &dword_81230,
          1);
        v1 = a1;
      }
    }
    ++v2;
    ++v3;
  }
  while ( v2 < 9 );
  result = v1[312];
  if ( result != 20 )
  {
    v17 = v1[313];
    if ( v17 != 0 )
      *(_DWORD *)(v17 + 20) = 1;
    result = v1[282];
    v18 = result - 1;
    if ( (unsigned int)(result - 1) <= 7 )
    {
      result = *((unsigned __int8 *)&jpt_12DB8 + v18);
      switch ( v18 )
      {
        case 0:
          result = sub_445F4(v1, 102);
          v1[313] = result;
          *(_DWORD *)(result + 20) = 0;
          break;
        case 1:
          result = sub_445F4(v1, 202);
          v1[313] = result;
          *(_DWORD *)(result + 20) = 0;
          break;
        case 2:
          result = sub_445F4(v1, 302);
          v1[313] = result;
          *(_DWORD *)(result + 20) = 0;
          break;
        case 3:
          result = sub_445F4(v1, 402);
          v1[313] = result;
          *(_DWORD *)(result + 20) = 0;
          break;
        case 4:
          result = sub_445F4(v1, 502);
          v1[313] = result;
          *(_DWORD *)(result + 20) = 0;
          break;
        case 5:
          return result;
        case 6:
          result = sub_445F4(v1, 12);
          v1[313] = result;
          *(_DWORD *)(result + 20) = 0;
          break;
        case 7:
          result = sub_445F4(v1, 22);
          v1[313] = result;
          *(_DWORD *)(result + 20) = 0;
          break;
        default:
          abort();
      }
    }
  }
  return result;
}

