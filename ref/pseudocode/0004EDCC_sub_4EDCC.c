/* ea:   0x0004EDCC */
/* name: sub_4EDCC */
/* size: 596 */
int __fastcall sub_4EDCC(_DWORD *a1)
{
  int v2; // r1
  int v3; // r10
  int v4; // r8
  int v5; // r9
  _DWORD *v6; // r7
  int v7; // r1
  int v8; // r0
  int (__fastcall *v9)(int, int, int, int, int); // r3
  int v10; // r0
  int v11; // r0
  int v12; // r4
  int result; // r0
  _DWORD v14[2]; // [sp+Ch] [bp-4Ch] BYREF
  _DWORD v15[4]; // [sp+14h] [bp-44h] BYREF
  _DWORD v16[13]; // [sp+24h] [bp-34h] BYREF

  v2 = a1[278];
  v14[0] = off_7181C;
  a1[278] = v2 + 1;
  if ( v2 == 3 )
    a1[278] = 0;
  sub_443FC(a1);
  v3 = 0;
  if ( (int)a1[277] > 0 )
  {
    v4 = v14[0];
    v5 = v14[0];
    do
    {
      v6 = &a1[v3];
      v7 = v6[268];
      v14[1] = v6[262];
      if ( v7 == 0 )
        __debugbreak();
      v8 = *(_DWORD *)(v7 + 12);
      if ( v8 == 16 )
      {
        v9 = sub_5C2CC;
      }
      else if ( v8 == 8 )
      {
        v9 = sub_5DBB0;
      }
      else
      {
        v9 = nullptr;
      }
      a1[122] = v9;
      switch ( v3 )
      {
        case 0:
          v4 = 20;
          v5 = 46;
          break;
        case 1:
          v4 = 73;
          v5 = 46;
          break;
        case 2:
          v4 = 128;
          v5 = 46;
          break;
        case 3:
          v10 = 20;
          goto LABEL_19;
        case 4:
          v10 = 73;
          goto LABEL_19;
        case 5:
          v10 = 128;
LABEL_19:
          v4 = v10;
          v5 = 146;
          break;
        default:
          break;
      }
      v11 = a1[278];
      v15[0] = 0;
      v12 = a1[1];
      v16[0] = v4;
      v15[2] = 40 * v11;
      v15[3] = 40 * v11 + 40;
      v16[1] = v4 + 25;
      v15[1] = 25;
      v16[3] = v5 + 40;
      v16[2] = v5;
      (*(void (__fastcall **)(int, _DWORD *, _DWORD, _DWORD *, int (__fastcall *)(int, int, int, int, int), _DWORD *, int))(*(_DWORD *)v12 + 20))(
        v12,
        v16,
        v6[268],
        v15,
        v9,
        v14,
        1);
      ++v3;
    }
    while ( v3 < a1[277] );
  }
  result = a1[274];
  switch ( result )
  {
    case 1:
      result = sub_44648(a1, 35, 88);
      break;
    case 2:
      result = sub_44648(a1, 88, 88);
      break;
    case 3:
      result = sub_44648(a1, 142, 88);
      break;
    case 4:
      result = sub_44648(a1, 35, 158);
      break;
    case 5:
      result = sub_44648(a1, 88, 158);
      break;
    case 6:
      result = sub_44648(a1, 142, 158);
      break;
    default:
      return result;
  }
  return result;
}

