/* ea:   0x0001A0B8 */
/* name: sub_1A0B8 */
/* size: 608 */
int __fastcall sub_1A0B8(int a1)
{
  int v2; // r0
  int v3; // r0
  int v4; // r0
  int v5; // r0
  int v6; // r0
  int v7; // r0
  int v8; // r0
  int result; // r0
  int v10; // r0
  int v11; // r0
  int v12; // r0

  switch ( *(_DWORD *)(a1 + 1028) )
  {
    case 1:
      v2 = *(_DWORD *)(a1 + 1052);
      if ( v2 != 0 )
      {
        *(_DWORD *)(v2 + 20) = 1;
        *(_DWORD *)(a1 + 1052) = 0;
      }
      return sub_44648(a1, 30);
    case 2:
      v3 = *(_DWORD *)(a1 + 1052);
      if ( v3 != 0 )
      {
        *(_DWORD *)(v3 + 20) = 1;
        *(_DWORD *)(a1 + 1052) = 0;
      }
      goto LABEL_31;
    case 3:
      v4 = *(_DWORD *)(a1 + 1052);
      if ( v4 != 0 )
      {
        *(_DWORD *)(v4 + 20) = 1;
        *(_DWORD *)(a1 + 1052) = 0;
      }
      return sub_44648(a1, 140);
    case 4:
      v5 = *(_DWORD *)(a1 + 1052);
      if ( v5 != 0 )
      {
        *(_DWORD *)(v5 + 20) = 1;
        *(_DWORD *)(a1 + 1052) = 0;
      }
      return sub_44648(a1, 30);
    case 5:
      v6 = *(_DWORD *)(a1 + 1052);
      if ( v6 != 0 )
      {
        *(_DWORD *)(v6 + 20) = 1;
        *(_DWORD *)(a1 + 1052) = 0;
      }
      goto LABEL_31;
    case 6:
      v7 = *(_DWORD *)(a1 + 1052);
      if ( v7 != 0 )
      {
        *(_DWORD *)(v7 + 20) = 1;
        *(_DWORD *)(a1 + 1052) = 0;
      }
      return sub_44648(a1, 140);
    case 7:
      v8 = *(_DWORD *)(a1 + 1052);
      if ( v8 != 0 )
      {
        *(_DWORD *)(v8 + 20) = 1;
        *(_DWORD *)(a1 + 1052) = 0;
      }
      return sub_44648(a1, 30);
    case 8:
      v10 = *(_DWORD *)(a1 + 1052);
      if ( v10 != 0 )
      {
        *(_DWORD *)(v10 + 20) = 1;
        *(_DWORD *)(a1 + 1052) = 0;
      }
      goto LABEL_31;
    case 9:
      v11 = *(_DWORD *)(a1 + 1052);
      if ( v11 != 0 )
      {
        *(_DWORD *)(v11 + 20) = 1;
        *(_DWORD *)(a1 + 1052) = 0;
      }
      return sub_44648(a1, 140);
    case 0xA:
      v12 = *(_DWORD *)(a1 + 1052);
      if ( v12 != 0 )
      {
        *(_DWORD *)(v12 + 20) = 1;
        *(_DWORD *)(a1 + 1052) = 0;
      }
LABEL_31:
      result = sub_44648(a1, 85);
      break;
    case 0xB:
      result = sub_445F4(a1, 1400);
      *(_DWORD *)(a1 + 1052) = result;
      *(_DWORD *)(result + 20) = 0;
      break;
    default:
      result = *(_DWORD *)(a1 + 1052);
      if ( result != 0 )
      {
        *(_DWORD *)(result + 20) = 1;
        *(_DWORD *)(a1 + 1052) = 0;
      }
      break;
  }
  return result;
}

