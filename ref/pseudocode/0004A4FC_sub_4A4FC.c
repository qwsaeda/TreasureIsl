/* ea:   0x0004A4FC */
/* name: sub_4A4FC */
/* size: 20 */
void __fastcall sub_4A4FC(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

