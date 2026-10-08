/* ea:   0x0005870C */
/* name: sub_5870C */
/* size: 80 */
int __fastcall sub_5870C(int result)
{
  *(_DWORD *)(result + 8) = 64;
  *(_DWORD *)result = off_73A48;
  *(_DWORD *)(result + 16) = 255;
  *(_DWORD *)(result + 4) = 0;
  *(_BYTE *)(result + 12) = 0;
  *(_DWORD *)(result + 20) = 44100;
  *(_DWORD *)(result + 24) = 128;
  *(_BYTE *)(result + 28) = 0;
  return result;
}

