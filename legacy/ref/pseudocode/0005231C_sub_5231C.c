/* ea:   0x0005231C */
/* name: sub_5231C */
/* size: 356 */
int __fastcall sub_5231C(int a1)
{
  int v2; // r0
  int result; // r0
  int v4; // r0
  int v5; // r0
  int v6; // r0

  switch ( *(_DWORD *)(a1 + 124) )
  {
    case 0:
      v6 = sub_53808(*(_DWORD *)(a1 + 24) + 52, *(_DWORD *)(a1 + 64) + 1, *(_DWORD *)(a1 + 68));
      if ( v6 != 0 && (*(_BYTE *)(v6 + 22) & 3) == 1 )
      {
        *(_BYTE *)(a1 + 128) = 2;
        *(_DWORD *)(a1 + 124) = 3;
      }
      goto LABEL_14;
    case 1:
      v5 = sub_53808(*(_DWORD *)(a1 + 24) + 52, *(_DWORD *)(a1 + 64), *(_DWORD *)(a1 + 68) - 1);
      if ( v5 == 0 || (*(_BYTE *)(v5 + 22) & 3) != 1 )
        goto LABEL_14;
      *(_BYTE *)(a1 + 129) = 0;
      *(_DWORD *)(a1 + 124) = 2;
      *(_BYTE *)(a1 + 128) = 1;
      *(_BYTE *)(a1 + 130) = -1;
      result = 255;
      break;
    case 2:
      v2 = sub_53808(*(_DWORD *)(a1 + 24) + 52, *(_DWORD *)(a1 + 64), *(_DWORD *)(a1 + 68) + 1);
      if ( v2 == 0 || (*(_BYTE *)(v2 + 22) & 3) != 1 )
        goto LABEL_14;
      *(_BYTE *)(a1 + 129) = 0;
      *(_DWORD *)(a1 + 124) = 1;
      *(_BYTE *)(a1 + 128) = 4;
      *(_BYTE *)(a1 + 130) = -1;
      result = 255;
      break;
    case 3:
      v4 = sub_53808(*(_DWORD *)(a1 + 24) + 52, *(_DWORD *)(a1 + 64) - 1, *(_DWORD *)(a1 + 68));
      if ( v4 == 0 || (*(_BYTE *)(v4 + 22) & 3) != 1 )
        goto LABEL_14;
      *(_DWORD *)(a1 + 124) = 0;
      *(_BYTE *)(a1 + 128) = 8;
      *(_BYTE *)(a1 + 129) = 0;
      *(_BYTE *)(a1 + 130) = -1;
      result = 255;
      break;
    default:
LABEL_14:
      *(_BYTE *)(a1 + 129) = 0;
      *(_BYTE *)(a1 + 130) = -1;
      result = 255;
      break;
  }
  return result;
}

