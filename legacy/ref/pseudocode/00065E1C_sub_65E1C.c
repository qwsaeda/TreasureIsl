/* ea:   0x00065E1C */
/* name: sub_65E1C */
/* size: 20 */
void __fastcall sub_65E1C(void **a1)
{
  void *v1; // r0

  v1 = *a1;
  if ( v1 != nullptr )
    free(v1);
}

